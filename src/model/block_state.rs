use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use super::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockState {
	#[serde(skip_serializing_if="Option::is_none")]
	variants: Option<HashMap<String, ModelType>>,
	#[serde(skip_serializing_if="Option::is_none")]
	multipart: Option<Vec<Multipart>>
}

impl Validate for BlockState {
	fn is_valid(&self) -> bool {
		self.variants.is_some() || self.multipart.is_some()
	}
}

impl Default for BlockState {
	fn default() -> BlockState {
		BlockState {
			variants: None,
			multipart: None,
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
	#[serde(skip_serializing_if="Option::is_none")]
	model: Option<String>,
	#[serde(skip_serializing_if="Option::is_none")]
	x: Option<u64>,
	#[serde(skip_serializing_if="Option::is_none")]
	y: Option<u64>,
	#[serde(skip_serializing_if="Option::is_none")]
	uvlock: Option<bool>,
	#[serde(skip_serializing_if="Option::is_none")]
	weight: Option<u64>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModelType {
	#[serde(skip_serializing)]
	Model(Model),
	#[serde(skip_serializing)]
	Models(Vec<Model>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Multipart {
	when: Option<HashMap<String, Case>>,
	apply: Option<ModelType>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Case {
	#[serde(skip_serializing)]
	State(String),
	#[serde(skip_serializing)]
	Or(Vec<HashMap<String, String>>)
}