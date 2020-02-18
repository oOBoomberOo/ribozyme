mod pack_meta;
mod namespace;
mod resource;
mod template;

pub mod resources {
	pub use super::resource::{Resource, ResourceError, ResourceFormat};
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
use std::io::{Read};
use std::iter::FromIterator;
use zip::ZipWriter;
use zip::write::FileOptions;
use console::style;
use serde_json as js;
impl Resourcepack {
	pub fn from_path(path: impl Into<PathBuf>) -> ProgramResult<Resourcepack> {
		let path: PathBuf = path.into();

		let packmeta = path.join("pack.mcmeta");
		let mut reader = fs::File::open(&packmeta)?;
		let mut content = String::default();
		reader.read_to_string(&mut content)?;

		// Workaround for BOM encoding format
		let bom_workaround = content.trim_start_matches('\u{feff}');

		let meta: Meta = match js::from_str(&bom_workaround) {
			Ok(result) => result,
			Err(error) => return Err(ProgramError::SerdeWithPath(packmeta, error))
		};

		let assets = path.join("assets");
		let namespaces = match assets.read_dir() {
			Ok(entries) => entries.filter_map(|entry| {
				match Namespace::from_entry(entry, &path) {
					Ok(namespace) => Some(namespace),
					Err(error) => {
						eprintln!("{} {}", style("[Warn]").yellow(), error);
						None
					}
				}
			}),
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

	pub fn build(self, output: impl Into<PathBuf>) -> ProgramResult<()> {
		let output: PathBuf = output.into();

		let writer = fs::File::create(&output)?;
		let mut zip = ZipWriter::new(writer);

		let options = FileOptions::default()
			.compression_method(get_compression_method())
			.unix_permissions(0o755);

		self.meta.build(&mut zip, options.clone())?;
		
		for namespace in self.assets {
			namespace.build(&mut zip, options.clone())?;
		}

		Ok(())
	}
}

use zip::CompressionMethod;
fn get_compression_method() -> CompressionMethod {
	if cfg!(feature = "deflate") {
		CompressionMethod::Deflated
	}
	else if cfg!(feature = "bzip2") {
		CompressionMethod::Bzip2
	}
	else {
		CompressionMethod::Stored
	}
}