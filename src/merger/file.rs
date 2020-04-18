use crate::rp::resource::{Resource, ResourceKind};
use anyhow::Result;
use serde_json as js;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::fs;
use std::io;

pub struct File {
	data: Vec<u8>,
	kind: ResourceKind,
}

impl File {
	pub fn new(data: Vec<u8>, kind: ResourceKind) -> File {
		File { data, kind }
	}

	pub fn from_resource(resource: Resource) -> Result<File> {
		let data = resource.data()?;
		let kind = resource.kind();
		let result = File::new(data, kind);
		Ok(result)
	}

	pub fn write(&self, path: PathBuf) -> io::Result<()> {
		if let Some(parent) = path.parent() {
			fs::create_dir_all(parent)?;
			fs::write(path, &self.data)?;
		}

		Ok(())
	}
}

impl fmt::Debug for File {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", String::from_utf8_lossy(&self.data))
	}
}

pub trait Merger {
	type Item;
	fn merge(left: Self::Item, right: Self::Item) -> Self::Item;
}

impl Merger for Result<File> {
	type Item = Result<File>;
	fn merge(left: Self::Item, right: Self::Item) -> Self::Item {
		let left = left?;
		let kind = left.kind;
		// TODO: Figure out why it need to do this
		let left = Ok(left);

		match kind {
			ResourceKind::Model => Model::merge(left, right),
			ResourceKind::Lang => Lang::merge(left, right),
			_ => right,
		}
	}
}

struct Model;

impl Merger for Model {
	type Item = Result<File>;
	fn merge(_left: Self::Item, right: Self::Item) -> Self::Item {
		right
	}
}

struct Lang;

impl Merger for Lang {
	type Item = Result<File>;
	fn merge(left: Self::Item, right: Self::Item) -> Self::Item {
		let left = left?;
		let right = right?;

		let original: HashMap<String, String> = js::from_slice(&left.data)?;
		let others: HashMap<String, String> = js::from_slice(&right.data)?;

		let result: HashMap<String, String> = original
			.into_iter()
			.chain(others.into_iter())
			.collect();

		let data = js::to_vec(&result)?;

		let result = File::new(data, left.kind);
		Ok(result)
	}
}
