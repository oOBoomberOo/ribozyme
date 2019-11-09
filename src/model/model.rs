use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;
use super::Validate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemModel {
	#[serde(skip_serializing_if="Option::is_none")]
	pub parent: Option<String>,
	#[serde(skip_serializing_if="Option::is_none")]
	pub display: Option<HashMap<String, Display>>,
	#[serde(skip_serializing_if="Option::is_none")]
	pub textures: Option<HashMap<String, String>>,
	#[serde(skip_serializing_if="Option::is_none")]
	pub elements: Option<Vec<Element>>,
	#[serde(skip_serializing_if="Option::is_none")]
	pub overrides: Option<Vec<Override>>
}

impl Default for ItemModel {
	fn default() -> ItemModel {
		ItemModel {
			parent: None,
			display: None,
			textures: None,
			elements: None,
			overrides: None
		}
	}
}

impl Validate for ItemModel {
	fn is_valid(&self) -> bool {
		self.parent.is_some() || self.elements.is_some() || self.textures.is_some() || self.display.is_some() || self.overrides.is_some()
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Display {
	#[serde(skip_serializing_if="Option::is_none")]
	rotation: Option<[f32; 3]>,
	#[serde(skip_serializing_if="Option::is_none")]
	translation: Option<[f32; 3]>,
	#[serde(skip_serializing_if="Option::is_none")]
	scale: Option<[f32; 3]>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Element {
	from: [f32; 3],
	to: [f32; 3],
	#[serde(skip_serializing_if="Option::is_none")]
	name: Option<String>,
	#[serde(skip_serializing_if="Option::is_none")]
	rotation: Option<Rotation>,
	#[serde(skip_serializing_if="Option::is_none")]
	shade: Option<bool>,
	#[serde(skip_serializing_if="Option::is_none")]
	faces: Option<HashMap<String, Face>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rotation {
	origin: [f32; 3],
	axis: String,
	angle: f32,
	#[serde(skip_serializing_if="Option::is_none")]
	rescale: Option<bool>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Face {
	#[serde(skip_serializing_if="Option::is_none")]
	uv: Option<[f32; 4]>,
	#[serde(skip_serializing_if="Option::is_none")]
	texture: Option<String>,
	#[serde(skip_serializing_if="Option::is_none")]
	cullface: Option<String>,
	#[serde(skip_serializing_if="Option::is_none")]
	rotation: Option<f32>,
	#[serde(skip_serializing_if="Option::is_none")]
	tintindex: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct Override {
	pub predicate: Predicate,
	pub model: String,
}

impl fmt::Display for Override {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", serde_json::to_string(&self).unwrap_or_default())
	}
}

impl Validate for Override {
	fn is_valid(&self) -> bool {
		self.predicate.is_valid()
	}
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct Predicate {
	#[serde(skip_serializing_if="Option::is_none")]
	pub angle: Option<f64>,
	#[serde(skip_serializing_if="Option::is_none")]
	pub blocking: Option<u64>,
	#[serde(skip_serializing_if="Option::is_none")]
	pub broken: Option<u64>,
	#[serde(skip_serializing_if="Option::is_none")]
	pub cast: Option<u64>,
	#[serde(skip_serializing_if="Option::is_none")]
	pub cooldown: Option<f64>,
	#[serde(skip_serializing_if="Option::is_none")]
	pub damage: Option<f64>,
	#[serde(skip_serializing_if="Option::is_none")]
	pub damaged: Option<u64>,
	#[serde(skip_serializing_if="Option::is_none")]
	pub lefthanded: Option<u64>,
	#[serde(skip_serializing_if="Option::is_none")]
	pub pull: Option<f64>,
	#[serde(skip_serializing_if="Option::is_none")]
	pub pulling: Option<u64>,
	#[serde(skip_serializing_if="Option::is_none")]
	pub throwing: Option<u64>,
	#[serde(skip_serializing_if="Option::is_none")]
	pub time: Option<f64>,
	#[serde(skip_serializing_if="Option::is_none")]
	pub custom_model_data: Option<i64>,
}

impl fmt::Display for Predicate {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", serde_json::to_string(&self).unwrap_or_default())
	}
}

impl Validate for Predicate {
	fn is_valid(&self) -> bool {
		self.angle.is_some() || self.blocking.is_some() || self.broken.is_some() || self.cast.is_some() || self.cooldown.is_some() || self.damage.is_some() || self.damaged.is_some() || self.lefthanded.is_some() || self.pull.is_some() || self.pulling.is_some() || self.throwing.is_some() || self.time.is_some() || self.custom_model_data.is_some()
	}
}