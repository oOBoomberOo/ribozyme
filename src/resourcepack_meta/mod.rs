use super::{ProgramError, ProgramResult, Resourcepack};
use std::path::PathBuf;
#[derive(Clone, Debug)]
pub struct ResourcepackMeta {
	location: PathBuf,
	name: String
}

use std::fs;
use std::io;
impl ResourcepackMeta {
	pub fn new(entry: Result<fs::DirEntry, io::Error>) -> ProgramResult<ResourcepackMeta> {
		let entry: fs::DirEntry = entry?;
		let location: PathBuf = entry.path();
		
		if location.is_file() {
			return Err(ProgramError::NotResourcepack(location));
		}
		else if location.is_dir() {
			let meta = location.join("pack.mcmeta");
			let assets = location.join("assets");
			if !meta.is_file() || !assets.is_dir() {
				return Err(ProgramError::NotResourcepack(location));
			}
		}

		let name = {
			let name = &location
			.file_name()
			.and_then(|osstr| osstr.to_str());
			match name {
				Some(x) => x,
				None => "Unknown Resourcepack"
			}
		};
		let name = name.to_owned();

		let result = ResourcepackMeta { location, name };
		Ok(result)
	}

	pub fn build(&self) -> ProgramResult<Resourcepack> {
		Resourcepack::from_path(&self.location)
	}
}

use std::fmt;
impl fmt::Display for ResourcepackMeta {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.name)
	}
}