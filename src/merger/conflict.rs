use crate::rp::Resource;
use super::file::{File, Merger};
use itertools::Itertools;
use anyhow::Result;
use anyhow::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct Conflict {
	pub conflicts: Vec<Resource>
}

impl Conflict {
	pub fn new(conflicts: Vec<Resource>) -> Conflict {
		Conflict { conflicts }
	}

	pub fn with_resource(resource: Resource) -> Conflict {
		Conflict::new(vec![resource])
	}

	pub fn add(&mut self, resource: Resource) {
		self.conflicts.push(resource);
	}

	pub fn solve(self) -> Result<File> {
		self.conflicts
			.into_iter()
			.map(File::from_resource)
			.fold1(Result::<File>::merge)
			.unwrap_or_else(|| Err(Error::msg("Empty iterator")))
	}
}