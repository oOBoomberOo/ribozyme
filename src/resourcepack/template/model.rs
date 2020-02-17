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
	elements: List<Element>
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
	uv: [f32; 4],
	texture: String,
	#[serde(skip_serializing_if="Option::is_none")]
	cullface: Option<String>,
	#[serde(skip_serializing_if="Option::is_none")]
	rotation: Option<i32>,
	#[serde(skip_serializing_if="Option::is_none")]
	tintindex: Option<i32>
}

impl Model {
	pub fn merge(self, other: Model) -> Model {
		let parent = other.parent;
		let ambient_occlusion = merge_ambient_occulsion(self.ambient_occlusion, other.ambient_occlusion);
		let display = merge_object(self.display, other.display);
		let textures = merge_object(self.textures, other.textures);
		let elements = merge_elements(self.elements, other.elements);
		
		Model { parent, ambient_occlusion, display, textures, elements }
	}
}

trait Mergable {
	fn merge(&self, other: Self) -> Self;
}

fn merge_ambient_occulsion(first: Option<bool>, second: Option<bool>) -> Option<bool> {
	if first.is_none() && second.is_none() {
		return None;
	}

	let result = first.unwrap_or_default() || second.unwrap_or_default();
	Some(result)
}

fn merge_object<T>(first: Object<T>, second: Object<T>) -> Object<T> {
	if first.is_none() && second.is_none() {
		return None;
	}

	let mut result = first.unwrap_or_default();
	let second = second.unwrap_or_default();

	result.extend(second);

	Some(result)
}

fn merge_elements(first: List<Element>, second: List<Element>) -> List<Element> {
	if first.is_none() && second.is_none() {
		return None;
	}

	let mut result = first.unwrap_or_default();
	let mut second = second.unwrap_or_default();
	result.append(&mut second);

	Some(result)
}
