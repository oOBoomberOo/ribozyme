use super::File;
use anyhow::{Context, Result};
use std::path::Path;
use superfusion::prelude::{Index, Relation};

pub struct Texture {
	data: Vec<u8>,
}

impl Texture {
	pub fn new(path: impl AsRef<Path>) -> Result<Self> {
		let path = path.as_ref();
		let data = std::fs::read(&path)
			.with_context(|| "Reading texture file")?;
		let result = Self { data };
		Ok(result)
	}
}

impl File for Texture {
	fn relation(&self) -> Vec<Relation> {
		vec![]
	}
	fn data(self) -> Vec<u8> {
		self.data
	}
	fn modify_relation(self, _: &Index, _: &Index) -> Self
	where
		Self: Sized,
	{
		self
	}
}
