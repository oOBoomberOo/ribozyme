mod chromosome;
mod nucleotide;

pub use chromosome::{Chromosome, ChromosomeKind};
pub use nucleotide::Nucleotide;

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
			conflicts: self.conflicts + rhs.conflicts,
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
	BlockState,
	Font,
	Other,
	Skip,
}

use std::fs;
use std::path::PathBuf;
pub use crate::model::{BlockState, Font, ItemModel, Validate};

pub struct Format {
	kind: FormatKind,
}

impl Format {
	pub fn identify_format(path: &PathBuf) -> Format {
		let path = path.to_owned();
		let mut result = Format::default();
		if let Some(extension) = path.extension() {
			if extension == "json" {
				let content = fs::read_to_string(&path).unwrap_or_default();
				let model: ItemModel = serde_json::from_str(&content).unwrap_or_default();
				let block_state: BlockState = serde_json::from_str(&content).unwrap_or_default();
				let font: Font = serde_json::from_str(&content).unwrap_or_default();

				if model.is_valid() {
					result.kind = FormatKind::Model;
				}
				else if block_state.is_valid() {
					result.kind = FormatKind::BlockState;
				}
				else if font.is_valid() {
					result.kind = FormatKind::Font;
				}
				else {
					result.kind = FormatKind::JSON;
				}
			}
		} else {
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
		Format {
			kind: FormatKind::Other,
		}
	}
}