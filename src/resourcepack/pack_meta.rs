use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Meta {
	pack: PackFormat,
	language: Option<HashMap<String, Language>>,
}

use serde_json::Value;
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct PackFormat {
	pack_format: u32,
	description: Value,
}

use std::fs;
use std::path::PathBuf;
use std::io::Write;
use zip::{ZipWriter, write::FileOptions};
use serde_json as js;
use crate::ProgramResult;
impl Meta {
	pub fn merge(&self, other: Meta) -> Meta {
		let pack = self.pack.merge(other.pack);
		let language = {
			if self.language.is_none() && other.language.is_none() {
				None
			} else {
				let mut language: HashMap<String, Language> = self.language.clone().unwrap_or_default();
				for (key, value) in other.language.unwrap_or_default() {
					let result = match language.get(&key) {
						Some(original) => original.merge(value),
						None => value,
					};
					language.insert(key, result);
				}

				Some(language)
			}
		};

		Meta { pack, language }
	}

	pub fn build(self, writer: &mut ZipWriter<fs::File>, options: FileOptions) -> ProgramResult<()> {
		writer.start_file_from_path(&PathBuf::from("pack.mcmeta"), options)?;

		let content = js::to_vec_pretty(&self)?;
		writer.write_all(&content)?;

		Ok(())
	}
}

use std::cmp::max;
impl PackFormat {
	fn merge(&self, other: PackFormat) -> PackFormat {
		let pack_format = max(self.pack_format, other.pack_format);
		let description = other.description;

		PackFormat {
			pack_format,
			description,
		}
	}
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct Language {
	name: String,
	region: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	bidirectional: Option<bool>,
}

impl Language {
	fn merge(&self, other: Language) -> Language {
		other
	}
}
