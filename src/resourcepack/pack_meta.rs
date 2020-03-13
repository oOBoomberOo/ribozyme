use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type Languages = Option<HashMap<String, Language>>;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Meta {
	pack: PackFormat,
	
	#[serde(skip_serializing_if="Option::is_none")]
	language: Languages
}

use serde_json::Value;
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct PackFormat {
	pack_format: u32,
	description: Value,
}

use std::fs;
use std::path::PathBuf;
use std::io::{Read};
use tar::Builder;
use flate2::write::GzEncoder;
use serde_json as js;
use crate::{ProgramResult, ProgramError};
use crate::utils::bom_fix;
use indicatif::ProgressBar;
impl Meta {
	pub fn from_path(path: PathBuf) -> ProgramResult<Meta> {
		let mut file = fs::File::open(&path)?;
		let mut content = String::default();
		file.read_to_string(&mut content)?;
		let content = bom_fix(content);

		let result: Meta = match js::from_str(&content) {
			Ok(result) => result,
			Err(error) => return Err(ProgramError::SerdeWithPath(path, error))
		};

		Ok(result)
	}

	pub fn merge(&self, other: Meta) -> Meta {
		let language = self.merge_language(&other);
		let pack = self.pack.merge(other.pack);

		Meta { pack, language }
	}

	fn merge_language(&self, other: &Meta) -> Languages {
		if self.language.is_none() && other.language.is_none() {
			None
		}
		else {
			let mut language = self.language.clone().unwrap_or_default();

			for (key, value) in other.language.clone().unwrap_or_default() {
				let (key, value) = match language.get(&key) {
					None => (key, value),
					Some(original) => (key, original.merge(value))
				};

				language.insert(key, value);
			}

			Some(language)
		}
	}

	pub fn build(self, writer: &mut Builder<GzEncoder<fs::File>>, progress_bar: &ProgressBar) -> ProgramResult<()> {
		{
			let mut file = fs::File::create("./pack.mcmeta")?;
			js::to_writer(&mut file, &self)?;
		}
		let path = PathBuf::from("pack.mcmeta");
		let mut reader = fs::File::open("./pack.mcmeta")?;
		
		fs::remove_file("./pack.mcmeta")?;
		progress_bar.inc(1);

		if let Err(error) = writer.append_file(&path, &mut reader) {
			return Err(ProgramError::IoWithPath(path, error));
		}

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
