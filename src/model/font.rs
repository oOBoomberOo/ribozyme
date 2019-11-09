use serde::{Serialize, Deserialize};
use serde_json::Value;
use super::Validate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Font {
	#[serde(skip_serializing_if="Option::is_none")]
	pub providers: Option<Vec<Value>>
}

impl Validate for Font {
	fn is_valid(&self) -> bool {
		self.providers.is_some()
	}
}

impl Default for Font {
	fn default() -> Font {
		Font {
			providers: None
		}
	}
}