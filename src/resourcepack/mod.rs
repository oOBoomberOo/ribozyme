mod pack_meta;
mod namespace;
mod resource;
mod template;

pub mod resources {
	pub use super::resource::{Resource, ResourceError};
}

use pack_meta::Meta;
use namespace::Namespace;
use resource::Resource;

use std::collections::HashSet;
#[derive(Default, Clone, Debug)]
pub struct Resourcepack {
	meta: Meta,
	assets: HashSet<Namespace>
}

use crate::{ProgramResult, ProgramError};
use std::path::PathBuf;
use std::fs;
use std::iter::FromIterator;
use serde_json as js;
impl Resourcepack {
	pub fn from_path(path: impl Into<PathBuf>) -> ProgramResult<Resourcepack> {
		let path: PathBuf = path.into();

		let packmeta = path.join("pack.mcmeta");
		let reader = match fs::File::open(&packmeta) {
			Ok(result) => result,
			Err(error) => return Err(ProgramError::IoWithPath(packmeta, error))
		};
		let meta: Meta = match js::from_reader(reader) {
			Ok(result) => result,
			Err(error) => return Err(ProgramError::Serde(packmeta, error))
		};

		let assets = path.join("assets");
		let namespaces = match assets.read_dir() {
			Ok(entries) => entries.filter_map(|entry| Namespace::from_entry(entry).ok()),
			Err(error) => return Err(ProgramError::IoWithPath(assets, error))
		};
			
		let assets: HashSet<Namespace> = HashSet::from_iter(namespaces);

		let result = Resourcepack { meta, assets };

		Ok(result)
	}

	pub fn merge(&mut self, other: Resourcepack) -> ProgramResult<()> {
		let meta = self.meta.merge(other.meta);
		let mut assets: HashSet<Namespace> = self.assets.clone();

		for namespace in other.assets {
			let result = match assets.take(&namespace) {
				Some(original) => original.merge(namespace)?,
				None => namespace
			};

			assets.insert(result);
		}

		self.meta = meta;
		self.assets = assets;

		Ok(())
	}
}