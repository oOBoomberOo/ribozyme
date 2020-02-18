use serde::{Serialize, Deserialize};
use std::collections::HashMap;

type Object<T> = Option<HashMap<String, T>>;
type List<T> = Option<Vec<T>>;
type Vector = Option<[f32; 3]>;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Model {
	#[serde(skip_serializing_if="Option::is_none")]
	parent: Option<String>,

	#[serde(skip_serializing_if="Option::is_none", rename = "ambientocclusion")]
	ambient_occlusion: Option<bool>,

	#[serde(skip_serializing_if="Option::is_none")]
	display: Object<Display>,

	#[serde(skip_serializing_if="Option::is_none")]
	textures: Object<String>,

	#[serde(skip_serializing_if="Option::is_none")]
	elements: List<Element>,

	#[serde(skip_serializing_if="Option::is_none")]
	gui_light: Option<String>,

	#[serde(skip_serializing_if="Option::is_none")]
	overrides: List<Override>
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct Display {
	#[serde(skip_serializing_if="Option::is_none")]
	rotation: Option<Vector>,

	#[serde(skip_serializing_if="Option::is_none")]
	translation: Option<Vector>,

	#[serde(skip_serializing_if="Option::is_none")]
	scale: Option<Vector>
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct Element {
	#[serde(skip_serializing_if="Option::is_none")]
	from: Vector,

	#[serde(skip_serializing_if="Option::is_none")]
	to: Vector,

	#[serde(skip_serializing_if="Option::is_none")]
	rotation: Option<Rotation>,

	#[serde(skip_serializing_if="Option::is_none")]
	shade: Option<bool>,

	#[serde(skip_serializing_if="Option::is_none")]
	faces: Object<Face>
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct Rotation {
	#[serde(skip_serializing_if="Option::is_none")]
	origin: Vector,

	#[serde(skip_serializing_if="Option::is_none")]
	axis: Option<String>,

	#[serde(skip_serializing_if="Option::is_none")]
	angle: Option<f32>,

	#[serde(skip_serializing_if="Option::is_none")]
	rescale: Option<bool>
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct Face {
	#[serde(skip_serializing_if="Option::is_none")]
	uv: Option<[f32; 4]>,

	texture: String,
	
	#[serde(skip_serializing_if="Option::is_none")]
	cullface: Option<String>,
	
	#[serde(skip_serializing_if="Option::is_none")]
	rotation: Option<i32>,
	
	#[serde(skip_serializing_if="Option::is_none")]
	tintindex: Option<i32>
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
struct Override {
	predicate: Predicate,
	model: String
}

type Float = Option<f32>;
type Int = Option<i32>;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct Predicate {
	angle: Float,
	blocking: Int,
	broken: Int,
	cast: Int,
	cooldown: Float,
	damage: Float,
	damaged: Int,
	lefthanded: Int,
	pull: Float,
	pulling: Int,
	throwing: Int,
	time: Float,
	custom_model_data: Int
}

impl Model {
	pub fn merge(self, other: Model) -> Model {
		let parent = other.parent;
		let ambient_occlusion = other.ambient_occlusion;
		let display = other.display;
		let textures = other.textures;
		let elements = other.elements;
		let gui_light = other.gui_light;
		let overrides = merge_overrides(self.overrides, other.overrides);
		
		Model { parent, ambient_occlusion, display, textures, elements, gui_light, overrides }
	}
}

trait Mergable {
	fn merge(&self, other: Self) -> Self;
}

fn merge_overrides(first: List<Override>, second: List<Override>) -> List<Override> {
	if first.is_none() && second.is_none() {
		return None;
	}

	let mut result: Vec<Override> = first.unwrap_or_default();
	let mut second: Vec<Override> = second.unwrap_or_default();

	result.append(&mut second);
	result.sort_by(|a, b| a.partial_cmp(b).unwrap());

	Some(result)
}

use std::cmp::{PartialOrd, Ordering};

impl PartialOrd for Override {
	fn partial_cmp(&self, other: &Override) -> Option<Ordering> {
		self.predicate.partial_cmp(&other.predicate)
	}
}

impl PartialEq for Predicate {
	fn eq(&self, other: &Predicate) -> bool {
		self.custom_model_data == other.custom_model_data
	}
}

impl PartialOrd for Predicate {
	fn partial_cmp(&self, other: &Predicate) -> Option<Ordering> {
		let first = self.custom_model_data.unwrap_or_default();
		let second = other.custom_model_data.unwrap_or_default();

		first.partial_cmp(&second)
	}
}