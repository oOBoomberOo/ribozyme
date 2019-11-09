mod chromosome;
mod nucleotide;
mod model;

pub use chromosome::{Chromosome, ChromosomeKind};
pub use nucleotide::Nucleotide;
pub use model::{ItemModel, Predicate, Validate, Invalid};

/**
 * Meta contain meta-data about conflicts and duplicates of nucleotide.
*/
#[derive(Debug)]
pub struct Meta {
	pub bytes: usize,
	pub nucleotides: usize,
	pub duplicates: usize,
	pub conflicts: usize,
}

impl Default for Meta {
	fn default() -> Self {
		Meta {
			bytes: 0,
			nucleotides: 0,
			duplicates: 0,
			conflicts: 0,
		}
	}
}

use std::ops::{Add, AddAssign};
impl Add for Meta {
	type Output = Self;

	fn add(self, rhs: Meta) -> Meta {
		Meta {
			bytes: self.bytes + rhs.bytes,
			nucleotides: self.nucleotides + rhs.nucleotides,
			duplicates: self.duplicates + rhs.duplicates,
			conflicts: self.conflicts + rhs.conflicts,
		}
	}
}

impl AddAssign for Meta {
	fn add_assign(&mut self, rhs: Meta) {
		*self = Self {
			bytes: self.bytes + rhs.bytes,
			nucleotides: self.nucleotides + rhs.nucleotides,
			duplicates: self.duplicates + rhs.duplicates,
			conflicts: self.conflicts + rhs.conflicts
		};
	}
}

/**
 * FormatKind define file format inside resourcepack
*/
#[derive(PartialEq, Eq, Debug)]
pub enum FormatKind {
	Model,
	JSON,
	Other,
	Skip,
}

use std::path::PathBuf;
use std::fs::File;

pub struct Format {
	kind: FormatKind
}

impl Format {
	pub fn identify_format(path: &PathBuf) -> Format  {
		let path = path.to_owned();
		let mut result = Format::default();
		if let Some(extension) = path.extension() {
			let content = match File::open(&path) {
				Ok(x) => x,
				_ => panic!("Unable to read file at {}", path.display())
			};

			if extension == "json" {
				result.kind = FormatKind::JSON;

				let model: ItemModel = match serde_json::from_reader(content) {
					Ok(content) => content,
					_ => ItemModel::invalid(),
				};

				if model.is_valid() {
					result.kind = FormatKind::Model;
				}
			}
		}
		else {
			if let Some(name) = path.file_name() {
				if name == ".DS_Store" {
					result.kind = FormatKind::Skip;
				}
			}
		}

		result
	}
}

impl Default for Format {
	fn default() -> Self {
		Format { kind: FormatKind::Other }
	}
}