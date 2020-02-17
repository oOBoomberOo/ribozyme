use std::collections::HashMap;
use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Meta {
	pack: PackFormat,
	language: HashMap<String, Language>
}

use serde_json::Value;
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct PackFormat {
	pack_format: u32,
	description: Value
}

impl Meta {
	pub fn merge(&self, other: Meta) -> Meta {
		let pack = self.pack.merge(other.pack);
		let mut language: HashMap<String, Language> = self.language.clone();
		for (key, value) in other.language {
			let result = match language.get(&key) {
				Some(original) => original.merge(value),
				None => value
			};

			language.insert(key, result);
		}
		
		Meta { pack, language }
	}
}

use std::cmp::max;
impl PackFormat {
	fn merge(&self, other: PackFormat) -> PackFormat {
		let pack_format = max(self.pack_format, other.pack_format);
		let description = other.description;

		PackFormat { pack_format, description }
	}
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct Language {
	name: String,
	region: String,
	#[serde(skip_serializing_if="Option::is_none")]
	bidirectional: Option<bool>
}

impl Language {
	fn merge(&self, other: Language) -> Language {
		other
	}
}