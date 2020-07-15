use super::File;
use superfusion::prelude::{Index, Relation};
use anyhow::{Result, Context};
use std::path::Path;

pub struct Other {
	data: Vec<u8>,
}

impl Other {
	pub fn new(path: impl AsRef<Path>) -> Result<Self> {
		let path = path.as_ref();
		let data = std::fs::read(&path)
			.with_context(|| format!("Fail to read file at {}", path.display()))?;
		let result = Self { data };
		Ok(result)
	}
}

impl File for Other {
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
