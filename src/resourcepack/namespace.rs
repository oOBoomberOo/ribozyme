use super::Resource;

use std::collections::HashSet;
use std::ffi::OsString;
#[derive(Default, Eq, Clone, Debug)]
pub struct Namespace {
	location: PathBuf,
	original: PathBuf,
	name: OsString,
	child: HashSet<Resource>
}

use crate::{ProgramResult, ProgramError};
use crate::resourcepack_meta::MetaPath;
use std::fs;
use std::io;
use std::iter::FromIterator;
use std::path::PathBuf;
use zip::ZipWriter;
use zip::write::FileOptions;
impl Namespace {
	pub fn from_entry(entry: io::Result<fs::DirEntry>, parent: &MetaPath) -> ProgramResult<Namespace> {
		let entry: fs::DirEntry = entry?;

		match entry.file_type() {
			Ok(result) => if result.is_file() {
				return Err(ProgramError::NotDirectory(entry.path()))
			},
			Err(error) => return Err(ProgramError::IoWithPath(entry.path(), error))
		};

		let name = entry.file_name();
		let path = entry.path();
		let location = path.strip_prefix(&parent.path)?.to_path_buf();
		let child_iter = match path.read_dir() {
			Ok(entries) => entries.filter_map(|entry| Resource::from_namespace(entry, &parent).ok()),
			Err(error) => return Err(ProgramError::IoWithPath(path, error))
		};
		let child: HashSet<Resource> = HashSet::from_iter(child_iter);
		let original = parent.original.join(&location);
		let result = Namespace { location, original, name, child };
		Ok(result)
	}

	pub fn merge(self, other: Namespace) -> ProgramResult<Namespace> {
		let location = other.location;
		let name = other.name;
		let original = other.original;
		let mut child: HashSet<Resource> = self.child;

		for resource in other.child {
			let result = match child.take(&resource) {
				Some(original) => original.merge(resource)?,
				None => resource
			};

			child.replace(result);
		}

		let result = Namespace { location, original, name, child };
		Ok(result)
	}

	pub fn build(self, zip: &mut ZipWriter<fs::File>, options: FileOptions) -> ProgramResult<()> {
		zip.add_directory_from_path(&self.location, options)?;

		for resource in self.child {
			resource.build(zip, options)?;
		}

		Ok(())
	}
}

use std::hash::{Hash, Hasher};
impl Hash for Namespace {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name.hash(state)
	}
}

impl PartialEq for Namespace {
	fn eq(&self, other: &Namespace) -> bool {
		self.name == other.name
	}
}

use std::fmt;
impl fmt::Display for Namespace {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self.name)
	}
}