use super::Resource;

use std::collections::HashSet;
use std::ffi::OsString;
#[derive(Default, Eq, Clone, Debug)]
pub struct Namespace {
	name: OsString,
	child: HashSet<Resource>
}

use crate::{ProgramResult, ProgramError};
use std::fs;
use std::io;
use std::iter::FromIterator;
impl Namespace {
	pub fn from_entry(entry: io::Result<fs::DirEntry>) -> ProgramResult<Namespace> {
		let entry: fs::DirEntry = entry?;

		match entry.file_type() {
			Ok(result) => if result.is_file() {
				return Err(ProgramError::NotDirectory(entry.path()))
			},
			Err(error) => return Err(ProgramError::IoWithPath(entry.path(), error))
		};

		let name = entry.file_name();
		let path = entry.path();
		let child_iter = match path.read_dir() {
			Ok(entries) => entries.filter_map(|entry| Resource::from_namespace(entry).ok()),
			Err(error) => return Err(ProgramError::IoWithPath(path, error))
		};
		let child: HashSet<Resource> = HashSet::from_iter(child_iter);

		let result = Namespace { name, child };
		Ok(result)
	}

	pub fn merge(self, other: Namespace) -> ProgramResult<Namespace> {
		let name = other.name;
		let mut child: HashSet<Resource> = self.child;

		for resource in other.child {
			let result = match child.take(&resource) {
				Some(original) => original.merge(resource)?,
				None => resource
			};

			child.insert(result);
		}

		let result = Namespace { name, child };
		Ok(result)
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