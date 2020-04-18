use super::{Conflict, File};
use std::collections::hash_map::IntoIter;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct ConflictSolver<K, V> {
	conflicts: HashMap<K, V>,
}

impl<K, V> ConflictSolver<K, V> {
	pub fn new(conflicts: HashMap<K, V>) -> ConflictSolver<K, V> {
		ConflictSolver { conflicts }
	}
}

impl IntoIterator for ConflictSolver<PathBuf, Conflict> {
	type Item = (PathBuf, File);
	type IntoIter = ConflictIter<PathBuf, Conflict>;
	fn into_iter(self) -> Self::IntoIter {
		let inner = self.conflicts.into_iter();
		ConflictIter::new(inner)
	}
}

pub struct ConflictIter<K, V> {
	inner: IntoIter<K, V>,
}

impl<K, V> ConflictIter<K, V> {
	pub fn new(inner: IntoIter<K, V>) -> ConflictIter<K, V> {
		ConflictIter { inner }
	}
}

impl Iterator for ConflictIter<PathBuf, Conflict> {
	type Item = (PathBuf, File);
	fn next(&mut self) -> Option<Self::Item> {
		self.inner
			.next()
			.map(|(key, conflicts): (PathBuf, Conflict)| (key, conflicts.solve()))
			.and_then(|(key, file)| file.map(|file| (key, file)))
	}
}
