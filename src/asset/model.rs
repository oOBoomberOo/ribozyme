use super::File;
use crate::namespace::{Kind, Namespace};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::convert::TryFrom;
use std::{collections::HashMap, path::{PathBuf, Path}};
use superfusion::prelude::{Error, Index, Pid, Relation};
use log::*;

pub struct Model {
	pid: Pid,
	path: PathBuf,
	data: ModelFormat,
}

impl Model {
	pub fn new(path: impl AsRef<Path>, pid: Pid) -> Result<Self> {
		let path = path.as_ref();
		let reader = std::fs::File::open(path)?;
		let data = serde_json::from_reader(reader)?;
		let result = Self { data, path: path.to_path_buf(), pid };
		Ok(result)
	}
}

impl File for Model {
	fn relation(&self) -> Vec<Relation> {
		let pid = self.pid;
		let mut result = vec![];

		if let Some(parent) = &self.data.parent {
			let parent = into_index(Kind::Model, parent, pid);
			result.push(parent);
		}

		if let Some(textures) = &self.data.textures {
			for texture in textures.0.values() {
				let index = into_index(Kind::Texture, texture, pid);
				result.push(index);
			}
		}

		if let Some(overrides) = &self.data.overrides {
			for model in &overrides.0 {
				let model = &model.model;
				let index = into_index(Kind::Model, model, pid);
				result.push(index);
			}
		}

		result.into_iter().map(Relation::new).collect()
	}
	fn data(self) -> Vec<u8> {
		serde_json::to_vec(&self.data).unwrap_or_default()
	}
	fn modify_relation(mut self, from: &Index, to: &Index) -> Self
	where
		Self: Sized,
	{
		let data = modify_relation(self.data, from, to);
		self.data = data;
		self
	}

	fn merge(self, mut other: Self) -> Result<Self, Error>
	where
		Self: Sized,
	{
		error!("Trying to merge '{}' with '{}'", self.path.display(), other.path.display());
		let data = self.data.merge(other.data);
		other.data = data;
		Ok(other)
	}
}

fn modify_relation(mut model: ModelFormat, from: &Index, to: &Index) -> ModelFormat {
	let from = match from_index(from) {
		Ok(v) => v,
		_ => return model,
	};
	let to = match from_index(to) {
		Ok(v) => v,
		_ => return model,
	};

	let set_if_true = |v: &mut Namespace| {
		if *v == from {
			*v = to.clone();
		}
	};

	if let Some(parent) = &mut model.parent {
		set_if_true(parent);
	}

	if let Some(textures) = &mut model.textures {
		textures.0.values_mut().for_each(set_if_true);
	}

	if let Some(overrides) = &mut model.overrides {
		overrides
			.0
			.iter_mut()
			.map(|i| &mut i.model)
			.for_each(set_if_true)
	}

	model
}

fn into_index(kind: Kind, namespace: &Namespace, pid: Pid) -> Index {
	let path = namespace.to_path(kind);
	Index::new(pid, path)
}

fn from_index(index: &Index) -> Result<Namespace> {
	let path = index.path();
	Namespace::try_from(path)
		.with_context(|| format!("'{}' cannot be converted to namespace", path.display()))
}

#[derive(Debug, Deserialize, Serialize)]
struct ModelFormat {
	#[serde(skip_serializing_if = "Option::is_none")]
	parent: Option<Namespace>,
	#[serde(skip_serializing_if = "Option::is_none")]
	ambientocclusion: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	display: Option<Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	textures: Option<Textures>,
	#[serde(skip_serializing_if = "Option::is_none")]
	elements: Option<Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	gui_light: Option<Side>,
	#[serde(skip_serializing_if = "Option::is_none")]
	overrides: Option<Overrides>,
}

impl ModelFormat {
	fn merge(self, other: Self) -> Self {
		let overrides = match (self.overrides, other.overrides) {
			(Some(v), None) => Some(v),
			(None, Some(v)) => Some(v),
			(Some(a), Some(b)) => Some(a.merge(b)),
			(None, None) => None,
		};

		Self {
			parent: other.parent,
			ambientocclusion: other.ambientocclusion,
			display: other.display,
			textures: other.textures,
			elements: other.elements,
			gui_light: other.gui_light,
			overrides,
		}
	}
}

#[derive(Debug, Deserialize, Serialize)]
struct Textures(HashMap<String, Namespace>);

#[derive(Debug, Deserialize, Serialize)]
enum Side {
	#[serde(rename = "front")]
	Front,
	#[serde(rename = "side")]
	Side,
}

#[derive(Debug, Deserialize, Serialize)]
struct Overrides(Vec<Override>);

impl Overrides {
	fn merge(self, mut other: Self) -> Self {
		let mut inner = self.0;
		inner.append(&mut other.0);
		inner.sort();
		Self(inner)
	}
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Override {
	predicate: Predicate,
	model: Namespace,
}

impl Eq for Override {}

impl PartialOrd for Override {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.predicate.partial_cmp(&other.predicate)
	}
}

impl Ord for Override {
	fn cmp(&self, other: &Self) -> Ordering {
		self.predicate
			.partial_cmp(&other.predicate)
			.unwrap_or_else(|| self.model.cmp(&other.model))
	}
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Predicate {
	#[serde(skip_serializing_if = "Option::is_none")]
	angle: Option<f64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	blocking: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	broken: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	cast: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	cooldown: Option<f64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	damage: Option<f64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	damaged: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	lefthand: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pull: Option<f64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pulling: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	charge: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	fireworks: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	throwing: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	time: Option<f64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	custom_model_data: Option<usize>,
}

impl PartialOrd for Predicate {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.custom_model_data.partial_cmp(&other.custom_model_data)
	}
}
