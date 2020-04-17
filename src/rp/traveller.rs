use super::{Resource, Source};
use std::fs;
use std::fs::DirEntry;
use std::io;
use std::path::PathBuf;
use anyhow::Result;

pub struct Traveller {
	root: PathBuf
}

impl Traveller {
	pub fn new(root: impl Into<PathBuf>) -> Traveller {
		let root = root.into();
		Traveller { root }
	}

	pub fn get_resources(&self, entry: io::Result<DirEntry>) -> Result<Vec<Resource>> {
		let entry = entry?;
		let path = entry.path();

		if path.is_dir() {
			let result = fs::read_dir(path)?
				.map(|entry| self.get_resources(entry))
				.flat_map(Result::ok)
				.flatten()
				.collect();
			Ok(result)
		} else {
			let parent = self.root.clone();
			let source = Source::from_parent(parent, path)?;
			let resource = Resource::new(source);

			let result = vec![resource];
			Ok(result)
		}
	}
}