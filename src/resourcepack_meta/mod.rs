use super::{ProgramResult, ResourcePath, Resourcepack};
use std::ffi::OsString;
use std::path::PathBuf;
#[derive(Clone, Debug)]
pub struct ResourcepackMeta {
	pub location: ResourcePath,
	name: OsString,
}

use std::fs;
use std::io;
use tempfile::tempdir;
impl ResourcepackMeta {
	pub fn new(entry: Result<fs::DirEntry, io::Error>) -> Result<ResourcepackMeta, MetaError> {
		let entry: fs::DirEntry = entry?;
		let location: PathBuf = entry.path();
		let name = match location.file_name() {
			Some(name) => name.to_os_string(),
			None => return Err(MetaError::InvalidName(location)),
		};

		if location.is_file() {
			// let reader = match fs::File::open(&location) {
			// 	Ok(result) => result,
			// 	Err(error) => return Err(MetaError::IoWithPath(location, error)),
			// };
			// let archive = Archive::new(GzDecoder::new(reader));

			let directory = tempdir()?;
			let location = ResourcePath::from_compress_file(directory.into_path(), location);
			let result = ResourcepackMeta { location, name };

			Ok(result)
		} else {
			let meta = location.join("pack.mcmeta");
			let assets = location.join("assets");
			if !meta.is_file() || !assets.is_dir() {
				return Err(MetaError::NotResourcepack(location));
			}

			let location = ResourcePath::from_directory(location);

			let result = ResourcepackMeta { location, name };
			Ok(result)
		}
	}

	pub fn build(self) -> ProgramResult<Resourcepack> {
		let result = Resourcepack::from_path(&self.location)?;
		self.location.remove()?;

		Ok(result)
	}

	pub fn get_name(&self) -> String {
		self.name.to_string_lossy().to_string()
	}
}

use std::fmt;
impl fmt::Display for ResourcepackMeta {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.get_name())
	}
}

#[derive(Debug)]
pub enum MetaError {
	InvalidName(PathBuf),
	Io(io::Error),
	IoWithPath(PathBuf, io::Error),
	NotResourcepack(PathBuf),
}

use colored::*;
impl fmt::Display for MetaError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			MetaError::InvalidName(path) => write!(
				f,
				"'{}' is not a valid path",
				path.display().to_string().cyan()
			),
			MetaError::Io(error) => write!(f, "{}", error),
			MetaError::IoWithPath(path, error) => {
				write!(f, "'{}' {}", path.display().to_string().cyan(), error)
			}
			MetaError::NotResourcepack(path) => write!(
				f,
				"'{}' is not a resourcepack",
				path.display().to_string().cyan()
			),
		}
	}
}

impl From<io::Error> for MetaError {
	fn from(error: io::Error) -> MetaError {
		MetaError::Io(error)
	}
}
