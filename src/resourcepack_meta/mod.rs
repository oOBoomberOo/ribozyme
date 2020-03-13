use super::{ProgramResult, ProgramError, ResourcePath, Resourcepack, catch_io_error};
use std::ffi::OsString;
use std::path::PathBuf;
#[derive(Clone, Debug)]
pub struct ResourcepackMeta {
	pub location: ResourcePath,
	name: OsString,
}

use std::fs;
use std::fs::File;
use std::io;
use tempfile::tempdir;
use tar::Archive;
use flate2::read::GzDecoder;
use std::borrow::Cow;
use std::path::Path;
impl ResourcepackMeta {
	pub fn new(entry: Result<fs::DirEntry, io::Error>) -> Result<ResourcepackMeta, MetaError> {
		let entry: fs::DirEntry = entry?;
		let location: PathBuf = entry.path();
		let name = match location.file_name() {
			Some(name) => name.to_os_string(),
			None => return Err(MetaError::InvalidName(location)),
		};

		if location.is_file() {
			if ResourcepackMeta::get_extension(&location) != Some("gz") {
				return Err(MetaError::NotResourcepack(location));
			}

			let pack_mcmeta = ResourcepackMeta::check_item(&location, "pack.mcmeta")?;
			if !pack_mcmeta {
				return Err(MetaError::NotResourcepack(location));
			}

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
		if self.location.origin.is_file() {
			self.extract_file()?;
		}
		let result = Resourcepack::from_path(&self.location)?;
		self.location.remove()?;

		Ok(result)
	}

	pub fn get_name(&self) -> String {
		self.name.to_string_lossy().to_string()
	}

	fn extract_file(&self) -> ProgramResult<()> {
		let reader = File::open(&self.location.origin)?;
		let mut archive = Archive::new(GzDecoder::new(reader));
		catch_io_error!(archive.unpack(&self.location.physical), self.location.physical.to_owned());
		Ok(())
	}

	fn get_extension(path: &PathBuf) -> Option<&str> {
		path.extension().and_then(|x| x.to_str())
	}

	fn check_item(location: &PathBuf, target: impl Into<PathBuf>) -> Result<bool, MetaError> {
		let target = target.into();
		let reader = File::open(&location)?;
		let mut archive = Archive::new(GzDecoder::new(reader));
		let result = archive.entries()?
			.filter_map(|entry| entry.ok())
			.any(|entry| -> bool {
				let path: Cow<Path> = match entry.path() {
					Ok(result) => result,
					Err(_) => return false,
				};
				path == target
			});
		Ok(result)
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_directory_extension() {
		assert_eq!(
			ResourcepackMeta::get_extension(&PathBuf::from("/test/path")),
			None
		);
	}

	#[test]
	fn get_file_extension() {
		assert_eq!(
			ResourcepackMeta::get_extension(&PathBuf::from("/test/path.txt")),
			Some("txt")
		);
	}
}