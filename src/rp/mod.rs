use std::collections::HashSet;
use std::path::PathBuf;
use anyhow::Result;
use std::fs::DirEntry;
use std::io;

mod source;
pub mod resource;
mod traveller;
pub use source::Source;
pub use resource::Resource;
pub use traveller::Traveller;

#[derive(Debug)]
pub struct Resourcepack {
	pub path: PathBuf,
	pub resource: HashSet<Resource>
}

impl Resourcepack {
	pub fn new(path: impl Into<PathBuf>, resource: HashSet<Resource>) -> Resourcepack {
		let path = path.into();
		Resourcepack { path, resource }
	}

	pub fn from_entry(entry: io::Result<DirEntry>) -> Result<Resourcepack> {
		let entry = entry?;
		let root = entry.path();
		let traveller = Traveller::new(&root);
		
		let resource = traveller.get_resources(Ok(entry))?;
		let resource: HashSet<Resource> = resource.into_iter().collect();
		let result = Resourcepack::new(root, resource);
		Ok(result)
	}
}