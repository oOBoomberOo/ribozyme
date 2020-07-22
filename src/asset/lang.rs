use super::File;
use anyhow::{Context, Result};
use log::*;
use std::fs;
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};
use superfusion::prelude::{Error, Index, Relation};

#[derive(Debug)]
pub struct Lang {
	path: PathBuf,
	data: HashMap<String, String>,
}

impl Lang {
	pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
		let path = path.into();
		let reader = fs::File::open(&path)
			.with_context(|| "Reading language file")?;
		let data = serde_json::from_reader(reader)
			.with_context(|| "Parsing language file")?;
		let result = Self { path, data };
		Ok(result)
	}
}

impl File for Lang {
	fn relation(&self) -> Vec<Relation> {
		vec![]
	}
	fn data(self) -> Vec<u8> {
		serde_json::to_vec(&self.data).unwrap_or_default()
	}
	fn modify_relation(self, _: &Index, _: &Index) -> Self
	where
		Self: Sized,
	{
		self
	}
	fn merge(mut self, other: Self) -> Result<Self, Error> {
		let file = &self.path;
		for (key, value) in other.data {
			if let Some(previous) = self.data.get(&key) {
				key_conflict(file, &key, &previous, &value);
			}

			self.data.insert(key, value);
		}

		Ok(self)
	}
}

fn key_conflict(file: &Path, key: &str, from: &str, to: &str) {
	warn!(
		"[{file}] Key '{key}' already exists. Replace {from:?} with {to:?}",
		file = file.display(),
		key = key,
		from = from,
		to = to,
	);
}
