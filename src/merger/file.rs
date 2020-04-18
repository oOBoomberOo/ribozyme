use crate::rp::resource::{Resource, ResourceKind};
use crate::rp::Source;
use crate::Style;
use anyhow::Result;
use serde_json as js;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

pub struct File {
	data: Vec<u8>,
	kind: ResourceKind,
	source: Source,
}

impl File {
	pub fn new(data: Vec<u8>, kind: ResourceKind, source: impl Into<Source>) -> File {
		let source = source.into();
		File { data, kind, source }
	}

	pub fn from_resource(resource: Resource) -> Result<File> {
		let data = resource.data()?;
		let kind = resource.kind();
		let source = resource.source;
		let result = File::new(data, kind, source);
		Ok(result)
	}

	pub fn write(&self, path: PathBuf) -> io::Result<()> {
		if let Some(parent) = path.parent() {
			fs::create_dir_all(parent)?;
			fs::write(path, &self.data)?;
		}

		Ok(())
	}

	pub fn origin(&self) -> PathBuf {
		self.source
			.surface_root()
			.unwrap_or_else(|| self.source.origin())
	}
}

impl fmt::Debug for File {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", String::from_utf8_lossy(&self.data))
	}
}

pub trait Merger {
	type Item;
	fn merge(left: Self::Item, right: Self::Item, style: Style) -> Self::Item;
}

impl Merger for Result<File> {
	type Item = Result<File>;
	fn merge(left: Self::Item, right: Self::Item, style: Style) -> Self::Item {
		let left = left?;
		let kind = left.kind;
		// TODO: Figure out why it need to do this
		let left = Ok(left);

		match kind {
			ResourceKind::Model => Model::merge(left, right, style),
			ResourceKind::Lang => Lang::merge(left, right, style),
			_ => right,
		}
	}
}

struct Model;

impl Merger for Model {
	type Item = Result<File>;
	fn merge(_left: Self::Item, right: Self::Item, _style: Style) -> Self::Item {
		right
	}
}

struct Lang;

impl Merger for Lang {
	type Item = Result<File>;
	fn merge(left: Self::Item, right: Self::Item, style: Style) -> Self::Item {
		let left = left?;
		let right = right?;

		let original: HashMap<String, String> = js::from_slice(&left.data)
			.or_else(|error| Err(FileError::Serde(left.origin(), error)))?;
		let others: HashMap<String, String> = js::from_slice(&right.data)
			.or_else(|error| Err(FileError::Serde(right.origin(), error)))?;

		let result: HashMap<String, String> =
			original.into_iter().chain(others.into_iter()).collect();

		let data = if style.pretty {
			js::to_vec_pretty(&result)
		} else {
			js::to_vec(&result)
		};

		let result = File::new(data?, left.kind, right.source);
		Ok(result)
	}
}

#[derive(Debug, Error)]
pub enum FileError {
	#[error("[{0}] {1}")]
	Serde(PathBuf, js::Error),
}
