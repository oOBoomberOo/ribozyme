use super::{ProgramError, ProgramResult, Resourcepack};
use std::path::PathBuf;
#[derive(Clone, Debug)]
pub struct ResourcepackMeta {
	location: MetaPath,
	name: String,
}

use std::fs;
use std::io;
use zip::ZipArchive;
use tempfile::tempdir;
impl ResourcepackMeta {
	pub fn new(entry: Result<fs::DirEntry, io::Error>) -> ProgramResult<ResourcepackMeta> {
		let entry: fs::DirEntry = entry?;
		let location: PathBuf = entry.path();
		let name = {
			let name = &location.file_name().and_then(|osstr| osstr.to_str());
			match name {
				Some(x) => (*x).to_owned(),
				None => "Unknown Resourcepack".to_owned(),
			}
		};

		if location.is_file() {
			let reader = match fs::File::open(&location) {
				Ok(result) => result,
				Err(error) => return Err(ProgramError::IoWithPath(location, error))
			};
			let mut zip = match ZipArchive::new(reader) {
				Ok(result) => result,
				Err(error) => return Err(ProgramError::ZipWithPath(location, error))
			};

			let packmeta = zip.by_name("pack.mcmeta").is_err();
			let assets = zip.by_name("assets").is_err();

			if packmeta || assets {
				return Err(ProgramError::NotResourcepack(location));
			}
			let directory = tempdir()?;
			for n in 0..zip.len() {
				let mut file = match zip.by_index(n) {
					Ok(result) => result,
					Err(error) => return Err(ProgramError::ZipWithPath(location, error))
				};
				let path = file.sanitized_name();
				let output = directory.path().join(&path);

				if file.is_dir() {
					fs::create_dir_all(&output)?;
				}
				else {
					if let Some(parent) = output.parent() {
						fs::create_dir_all(parent)?;
					}

					let mut writer = fs::File::create(output)?;
					io::copy(&mut file, &mut writer)?;
				}
			}

			let location = MetaPath::new(directory.into_path(), true);
			let result = ResourcepackMeta { location, name };

			Ok(result)
		}
		else {
			let meta = location.join("pack.mcmeta");
			let assets = location.join("assets");
			if !meta.is_file() || !assets.is_dir() {
				return Err(ProgramError::NotResourcepack(location));
			}

			let location = MetaPath::new(location, false);

			let result = ResourcepackMeta { location, name };
			Ok(result)
		}
	}

	pub fn build(self) -> ProgramResult<Resourcepack> {
		let result = Resourcepack::from_path(&self.location.path)?;
		self.location.remove()?;

		Ok(result)
	}
}

use std::fmt;
impl fmt::Display for ResourcepackMeta {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.name)
	}
}

#[derive(Clone, Debug)]
struct MetaPath {
	path: PathBuf,
	is_temp: bool,
}

impl MetaPath {
	fn new(path: impl Into<PathBuf>, is_temp: bool) -> MetaPath {
		let path = path.into();
		MetaPath { path, is_temp }
	}

	fn remove(self) -> ProgramResult<()> {
		if self.is_temp {
			fs::remove_dir_all(self.path)?;
		}

		Ok(())
	}
}
