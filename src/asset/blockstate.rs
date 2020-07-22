use super::{from_index, into_index, File, Kind};
use crate::namespace::Namespace;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use superfusion::prelude::{Index, Pid, Relation};

pub struct BlockState {
	pid: Pid,
	data: BlockstateFormat,
}

impl BlockState {
	pub fn new(path: impl AsRef<Path>, pid: Pid) -> Result<Self> {
		let reader = std::fs::File::open(path)
			.with_context(|| "Reading blockstate file")?;
		let data = serde_json::from_reader(reader)
			.with_context(|| "Parsing blockstate file")?;
		let result = Self { data, pid };
		Ok(result)
	}
}

impl File for BlockState {
	fn relation(&self) -> Vec<Relation> {
		self.data
			.relation()
			.iter()
			.map(|namespace| into_index(Kind::Model, namespace, self.pid))
			.map(Relation::new)
			.collect()
	}
	fn data(self) -> Vec<u8> {
		serde_json::to_vec(&self.data).unwrap_or_default()
	}

	fn modify_relation(mut self, from: &Index, to: &Index) -> Self
	where
		Self: Sized,
	{
		let data = self.data.modify(from, to);
		self.data = data;
		self
	}
}

macro_rules! get_or {
	($self:ident, $x:expr) => {
		match $x {
			Ok(v) => v,
			Err(_) => return $self,
			}
	};
}

#[derive(Debug, Deserialize, Serialize)]
enum BlockstateFormat {
	#[serde(rename = "variants")]
	Variant(Box<VariantFormat>),
	#[serde(rename = "multipart")]
	Multipart(Box<MultipartFormat>),
}

impl BlockstateFormat {
	fn modify(self, from: &Index, to: &Index) -> Self {
		let from = get_or![self, from_index(from)];
		let to = get_or![self, from_index(to)];
		self.replace(from, to)
	}

	fn replace(mut self, from: Namespace, to: Namespace) -> Self {
		let namespaces = match &mut self {
			Self::Variant(variant) => variant.namespace(),
			Self::Multipart(multipart) => multipart.namespace(),
		};
		namespaces
			.into_iter()
			.filter(|v| *v == &from)
			.for_each(|v| *v = to.clone());

		self
	}

	fn relation(&self) -> Vec<&Namespace> {
		let models = match self {
			Self::Variant(variant) => variant.models(),
			Self::Multipart(multipart) => multipart.models(),
		};

		models.iter().map(|f| &f.model).collect()
	}
}

#[derive(Debug, Deserialize, Serialize)]
struct VariantFormat(HashMap<String, ModelFormat>);

impl VariantFormat {
	fn models(&self) -> Vec<&Model> {
		self.0.values().flat_map(ModelFormat::models).collect()
	}

	fn namespace(&mut self) -> Vec<&mut Namespace> {
		self.0
			.values_mut()
			.flat_map(ModelFormat::namespace)
			.collect()
	}
}

#[derive(Debug, Deserialize, Serialize)]
struct MultipartFormat(HashMap<String, CaseFormat>);

impl MultipartFormat {
	fn models(&self) -> Vec<&Model> {
		self.0
			.values()
			.map(|case| &case.apply)
			.flat_map(ModelFormat::models)
			.collect()
	}

	fn namespace(&mut self) -> Vec<&mut Namespace> {
		self.0
			.values_mut()
			.map(|case| &mut case.apply)
			.flat_map(ModelFormat::namespace)
			.collect()
	}
}

#[derive(Debug, Deserialize, Serialize)]
struct CaseFormat {
	#[serde(skip_serializing_if = "Option::is_none")]
	when: Option<Value>,
	apply: ModelFormat,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum ModelFormat {
	Single(Model),
	Multiple(Vec<Model>),
}

impl ModelFormat {
	fn models(&self) -> Vec<&Model> {
		match &self {
			Self::Single(model) => vec![model],
			Self::Multiple(models) => models.iter().collect(),
		}
	}

	fn namespace(&mut self) -> Vec<&mut Namespace> {
		match self {
			Self::Single(model) => vec![model.namespace()],
			Self::Multiple(models) => models.iter_mut().map(Model::namespace).collect(),
		}
	}
}

#[derive(Debug, Deserialize, Serialize)]
struct Model {
	model: Namespace,
	#[serde(skip_serializing_if = "Option::is_none")]
	x: Option<i32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	y: Option<i32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	uvlock: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	weight: Option<f32>,
}

impl Model {
	fn namespace(&mut self) -> &mut Namespace {
		&mut self.model
	}
}
