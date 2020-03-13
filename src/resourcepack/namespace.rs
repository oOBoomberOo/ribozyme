use super::Resource;

use std::collections::HashSet;
use std::ffi::OsString;
#[derive(Default, Eq, Clone, Debug)]
pub struct Namespace {
	location: ResourcePath,
	relative: PathBuf,
	name: OsString,
	child: HashSet<Resource>
}

use crate::{ProgramResult, ProgramError, ResourcePath};
use super::resource::ResourceError;
use std::fs;
use std::io;
use std::path::{PathBuf, Path};
use std::iter::FromIterator;
use rayon::prelude::*;
use indicatif::ProgressBar;
impl Namespace {
	pub fn from_entry(entry: io::Result<fs::DirEntry>, root: &ResourcePath) -> ProgramResult<Namespace> {
		let entry: fs::DirEntry = entry?;
		let path = entry.path();

		match entry.file_type() {
			Ok(result) => if result.is_file() { return Err(ResourceError::IgnoredFile.into()) },
			Err(error) => return Err(ProgramError::IoWithPath(path, error))
		};

		let name = entry.file_name();
		let relative = path.strip_prefix(&root.physical)?.to_path_buf();
		let location = root.join(&relative);

		let child_iter = match path.read_dir() {
			Ok(entries) => entries,
			Err(error) => return Err(ProgramError::IoWithPath(path, error))
		};
		let child: Result<Vec<Resource>, _> = child_iter
			.par_bridge()
			.map(|entry| Resource::from_namespace(entry, &root))
			.filter_map(|resource| match resource {
				Err(ProgramError::Resource(ResourceError::IgnoredFile)) => None,
				other => Some(other)
			})
			.collect();

		let child: HashSet<Resource> = HashSet::from_iter(child?);

		let result = Namespace { location, relative, name, child };
		Ok(result)
	}

	pub fn merge(self, other: Namespace, progress_bar: &ProgressBar) -> ProgramResult<Namespace> {
		let location = self.location;
		let relative = self.relative;
		let name = other.name;
		let mut child: HashSet<Resource> = self.child;

		for resource in other.child {
			let resource = match child.take(&resource) {
				None => Ok(resource),
				Some(original) => original.merge(resource, progress_bar)
			};
			child.replace(resource?);
		}

		progress_bar.inc(child.len() as u64);

		let result = Namespace { location, relative, name, child };
		Ok(result)
	}

	pub fn build(self, path: &Path, progress_bar: &ProgressBar) -> ProgramResult<()> {
		
		let output = path.join(&self.relative);
		fs::create_dir_all(&output)?;
		progress_bar.set_message(&self.relative.display().to_string());
		progress_bar.inc(1);

		self.child
			.into_iter()
			.try_for_each(|resource| resource.build(path, progress_bar))
	}

	pub fn count(&self) -> u64 {
		self.child.iter().fold(0, |acc, resource| acc + resource.count()) + 1
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