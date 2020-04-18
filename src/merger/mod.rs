use std::path::PathBuf;
use std::collections::HashMap;
use crate::rp::Resourcepack;
use std::fs;
use anyhow::Result;

mod conflict;
mod conflict_solver;
mod file;
pub use file::File;
pub use conflict::Conflict;
pub use conflict_solver::ConflictSolver;

#[derive(Debug)]
pub struct Merger {
	resourcepacks: Vec<Resourcepack>,
}

impl Merger {
	pub fn new(resourcepacks: Vec<Resourcepack>) -> Merger {
		Self { resourcepacks }
	}

	pub fn get_conflict(self) -> HashMap<PathBuf, Conflict> {
		let mut result: HashMap<PathBuf, Conflict> = HashMap::default();

		self.resourcepacks
			.into_iter()
			.flat_map(|x| x.resource)
			.for_each(|x| x.into_conflict(&mut result));

		result
	}

	pub fn into_conflict_solver(self) -> ConflictSolver {
		let conflicts = self.get_conflict();
		ConflictSolver::new(conflicts)
	}

	pub fn from_path(path: impl Into<PathBuf>) -> Result<Merger> {
		let resourcepacks = fs::read_dir(path.into())?
			.map(Resourcepack::from_entry)
			.filter_map(Result::ok)
			.collect();
		let result = Merger::new(resourcepacks);
		Ok(result)
	}
}
