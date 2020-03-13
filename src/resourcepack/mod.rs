mod pack_meta;
mod namespace;
mod resource;
mod template;

pub mod resources {
	pub use super::resource::{Resource, ResourceError, ResourceFormat};
}

use crate::{ProgramResult, ProgramError, ResourcePath};
use pack_meta::Meta;
use namespace::Namespace;
use resource::{Resource, ResourceError};

use std::collections::HashSet;
#[derive(Clone, Debug)]
pub struct Resourcepack {
	location: ResourcePath,
	meta: Meta,
	assets: HashSet<Namespace>
}

use std::path::PathBuf;
use std::fs::File;
use std::iter::FromIterator;
use tar::Builder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rayon::prelude::*;
use indicatif::ProgressBar;
impl Resourcepack {
	pub fn new(location: &PathBuf) -> Resourcepack {
		let location = ResourcePath::from_directory(location);
		let meta = Meta::default();
		let assets = HashSet::default();
		Resourcepack { location, meta, assets }
	}

	pub fn from_path(resource_path: &ResourcePath) -> ProgramResult<Resourcepack> {
		let path = &resource_path.physical;

		let pack_meta = path.join("pack.mcmeta");
		let meta = Meta::from_path(pack_meta)?;

		let assets = path.join("assets");
		let namespaces = match assets.read_dir() {
			Ok(entries) => entries,
			Err(error) => return Err(ProgramError::IoWithPath(assets, error))
		};

		let namespaces: Result<Vec<Namespace>, _> = namespaces
			.par_bridge()
			.map(|entry| Namespace::from_entry(entry, &resource_path))
			.filter_map(|resource| match resource {
				Err(ProgramError::Resource(ResourceError::IgnoredFile)) => None,
				other => Some(other)
			})
			.collect();
		
		let location = resource_path.to_owned();
		let assets: HashSet<Namespace> = HashSet::from_iter(namespaces?);

		let result = Resourcepack { location, meta, assets };

		Ok(result)
	}

	pub fn merge(&mut self, other: Resourcepack, progress_bar: &ProgressBar) -> ProgramResult<()> {
		let meta = self.meta.merge(other.meta);
		let mut assets: HashSet<Namespace> = self.assets.clone();

		other.assets
			.into_iter()
			.map(|namespace| {
				match self.assets.take(&namespace) {
					Some(original) => original.merge(namespace, progress_bar),
					None => Ok(namespace)
				}
			})
			.try_for_each(|namespace| -> ProgramResult<()> {
				assets.replace(namespace?);
				Ok(())
			})?;

		progress_bar.inc(assets.len() as u64);

		self.meta = meta;
		self.assets = assets;

		Ok(())
	}

	pub fn build(self, progress_bar: &ProgressBar, compression_level: u32) -> ProgramResult<()> {
		let output = self.location.physical;

		let writer = File::create(&output)?;
		let mut archive = Builder::new(GzEncoder::new(writer, Compression::new(compression_level)));

		self.meta.build(&mut archive, progress_bar)?;
		
		let result: Result<(), _> = self.assets
			.into_iter()
			.try_for_each(|namespace| namespace.build(&mut archive, progress_bar));
		
		if let Err(error) = result {
				archive.finish()?;
				// fs::remove_dir_all(output)?;
			
				return Err(error);
		}
		archive.finish()?;
			
		Ok(())
	}

	pub fn count(&self) -> u64 {
		self.assets.iter().fold(0, |acc, namespace| acc + namespace.count())
	}
}