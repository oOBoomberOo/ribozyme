use crate::Style;
use super::{Conflict, File};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;
use rayon::prelude::*;

pub struct ConflictSolver {
	conflicts: HashMap<PathBuf, Conflict>,
}

impl ConflictSolver {
	pub fn new(conflicts: HashMap<PathBuf, Conflict>) -> ConflictSolver {
		ConflictSolver { conflicts }
	}

	pub fn solve(self, style: Style) -> Result<HashMap<PathBuf, File>> {
		self.conflicts
			.into_par_iter()
			// Solve conflict
			.map(|(key, conflicts)| (key, conflicts.solve(style)))
			// Transform tuple of "key" and "file result" into result of tuple
			.map(|(key, file)| file.map(|f| (key, f)))
			.collect()
	}
}