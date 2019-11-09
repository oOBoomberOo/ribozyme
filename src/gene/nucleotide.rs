use super::{Chromosome, Format, FormatKind, ItemModel, Meta};
use colored::*;
use std::fs;
use std::fs::File;
use std::io::Result as iResult;
use std::io::Write;
use std::path::{Path, PathBuf};

/**
 * Contain genetic data of a file
*/
pub struct Nucleotide {
	location: PathBuf,
	ribosome: PathBuf,
}

impl Nucleotide {
	pub fn new(location: &PathBuf, chromosome: &Chromosome) -> Nucleotide {
		let location = location.to_owned();
		let ribosome = Nucleotide::helicase(&location, &chromosome);
		Nucleotide { location, ribosome }
	}

	/**
	 * Handle merging of nucleotide
	*/
	pub fn handle(&self) -> Meta {
		let mut meta = Meta::default();

		//* Check if output file already exists
		if self.ribosome.exists() {
			meta += self.duplicate().unwrap_or_default();
		} else {
			meta.bytes += self.transcript().unwrap_or_default();
		}

		meta
	}

	/**
	 * Handle duplicate nucleotides
	*/
	fn duplicate(&self) -> iResult<Meta> {
		let mut meta: Meta = Meta::default();
		let format = Format::identify_format(&self.location);

		//* Completely replace nucleotide
		if format.kind == FormatKind::Other {
			meta.bytes += self.transcript().unwrap_or_default();
			meta.duplicates += 1;
		}
		//* Merge `overrides` object and replace everything else
		else if format.kind == FormatKind::Model {
			//* Read and store JSON data of "current file" and "new file" into `ItemModel` struct
			let prev_content = fs::read_to_string(&self.ribosome)?;
			let prev: ItemModel = serde_json::from_str(&prev_content)?;
			let next_content = fs::read_to_string(&self.location)?;
			let next: ItemModel = serde_json::from_str(&next_content)?;

			//* Clone `next` to give it more priority over `prev`
			let mut result: ItemModel = next.clone();
			let mut overrides = Vec::default();
			let mut conflicts = Vec::default();

			/*
			 * 1. Loop over `next.overrides`
			 * 2. For each instance, loop over `prev.overrides`
			 * 3. Check if `next.predicate` == `prev.predicate`
			 * 4. If true push them to `conflicts`
			*/
			if let Some(next_overrides) = next.overrides.clone() {
				if let Some(prev_overrides) = prev.overrides.clone() {
					for i in &next_overrides {
						for j in &prev_overrides {
							if &i.predicate == &j.predicate && &i.model != &j.model {
								conflicts.push((i.clone(), j.clone()));
							}
						}
					}
				}
			}

			//* Print all conflicts
			for (i, j) in &conflicts {
				println!(
					"{} Found conflicted predicate: {} <-> {} in {}",
					"[!]".red(),
					format!("{}", i).green(),
					format!("{}", j).green(),
					format!("'{}'", self.location.display()).cyan()
				);
			}

			//* Append all overrides (from both `prev` and `next`) to `overrides`
			if let Some(mut prev_overrides) = prev.overrides {
				overrides.append(&mut prev_overrides);
			}
			if let Some(mut next_overrides) = next.overrides {
				overrides.append(&mut next_overrides);
			}

			//* Sort overrides because minecraft can't fucking handle it
			overrides.sort_by(|a, b| {
				a.predicate
					.custom_model_data
					.cmp(&b.predicate.custom_model_data)
			});

			result.overrides = Some(overrides);

			//* Convert JSON data to string
			let content = serde_json::to_string_pretty(&result)?;
			meta.bytes += self.transport(&content)?;
			meta.conflicts += conflicts.len();
			meta.duplicates += 1;
		}
		//* Merge JSON object
		else if format.kind == FormatKind::JSON {
			let prev_content = fs::read_to_string(&self.ribosome)?;
			let prev: Value = serde_json::from_str(&prev_content)?;
			let next_content = fs::read_to_string(&self.location)?;
			let next: Value = serde_json::from_str(&next_content)?;
			let mut result = prev;
			merge_json(&mut result, next);

			let content = serde_json::to_string_pretty(&result)?;
			meta.bytes += self.transport(&content)?;
			meta.duplicates += 1;
		}

		Ok(meta)
	}

	/**
	 * Clone this nucleotide to output ribosome
	*/
	fn transcript(&self) -> iResult<usize> {
		let parent = self.ribosome_complex();
		fs::create_dir_all(parent)?;

		let mut reader = File::open(&self.location)?;
		let mut writer = File::create(&self.ribosome)?;
		let bytes = std::io::copy(&mut reader, &mut writer)? as usize;
		Ok(bytes)
	}

	/**
	 * Transport `content` to ribosome and produce amino acids/protein
	*/
	fn transport(&self, content: &str) -> iResult<usize> {
		let parent = self.ribosome_complex();
		fs::create_dir_all(parent)?;

		let mut file = File::create(&self.ribosome)?;
		let bytes = file.write(content.as_bytes())?;
		Ok(bytes)
	}

	/**
	 * Return the location of output ribosome's parent
	*/
	fn ribosome_complex(&self) -> &Path {
		match self.ribosome.parent() {
			Some(parent) => parent,
			None => panic!("Unable to get location of Ribosome Complex"),
		}
	}

	/**
	 * It's quite close to the actual helicase/RNA translation actually  
	 *   
	 * 1. Unzip DNA  
	 * 2. Copy DNA strand data to RNA  
	 * 3. Transport RNA to Ribosome  
	 * 4. Return location of said ribosome
	*/
	fn helicase(nucleotide: &PathBuf, chromosome: &Chromosome) -> PathBuf {
		let rna = match nucleotide.to_str() {
			Some(rna) => rna,
			None => panic!("Unable to copy DNA to RNA"),
		};
		let ribosome = match chromosome.output.to_str() {
			Some(ribosome) => ribosome,
			None => panic!("Unable to find Ribosome location"),
		};
		let chromosome = match chromosome.location.to_str() {
			Some(chromosome) => chromosome,
			None => panic!("Unable to convert chromosome to &str"),
		};
		let result = rna.replace(&chromosome, &ribosome);
		PathBuf::from(result)
	}
}

use serde_json::Value;
/**
 * It's merge JSON data, what else do you expect?
*/
fn merge_json(a: &mut Value, b: Value) {
	match (a, b) {
		(a @ &mut Value::Object(_), Value::Object(b)) => {
			let a = a.as_object_mut().unwrap();
			for (k, v) in b {
				merge_json(a.entry(k).or_insert(Value::Null), v);
			}
		}
		(a, b) => *a = b,
	}
}
