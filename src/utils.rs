use std::path::{PathBuf, Path};

#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct ResourcePath {
	pub physical: PathBuf,
	pub origin: PathBuf,
	is_temp: bool
}

use std::fs;
use std::io;
impl ResourcePath {
	pub fn from_directory(path: impl Into<PathBuf>) -> ResourcePath {
		let path = path.into();
		let physical = path.clone();
		let origin = path;
		ResourcePath { physical, origin, is_temp: false }
	}

	pub fn from_compress_file(physical: PathBuf, origin: PathBuf) -> ResourcePath {
		ResourcePath { physical, origin, is_temp: true }
	}

	pub fn remove(self) -> io::Result<()> {
		if self.is_temp {
			fs::remove_dir_all(self.physical)?;
		}

		Ok(())
	}

	pub fn join(&self, path: impl AsRef<Path>) -> ResourcePath {
		let physical = self.physical.join(&path);
		let origin = self.origin.join(path);
		ResourcePath { physical, origin, is_temp: false }
	}
}

/// Serde CAN'T deserialize file with BOM in front so we have to do this first
pub fn bom_fix(content: String) -> String {
	content.trim_start_matches('\u{feff}').to_owned()
}

pub fn bom_fix_vec(content: &[u8]) -> &[u8] {
	if content[0] == 0xEF {
		&content[3..]
	}
	else {
		content
	}
}