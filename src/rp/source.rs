use std::path::{PathBuf, Path};
use std::path::StripPrefixError;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct Source {
	pub relative: PathBuf,
	pub root: PathBuf
}

impl Source {
	pub fn new(relative: impl Into<PathBuf>, root: impl Into<PathBuf>) -> Source {
		let relative = relative.into();
		let root = root.into();
		Source { relative, root }
	}

	#[cfg(test)]
	pub fn test(relative: impl Into<PathBuf>) -> Source {
		Source::new(relative, PathBuf::default())
	}

	pub fn from_parent(parent: impl AsRef<Path>, origin: impl Into<PathBuf>) -> Result<Source, StripPrefixError> {
		let root = parent.as_ref().to_path_buf();
		let origin = origin.into();
		let relative = origin.strip_prefix(parent).map(Path::to_path_buf)?;
		let result = Source::new(relative, root);
		
		Ok(result)
	}

	pub fn origin(&self) -> PathBuf {
		self.root.join(&self.relative)
	}

	pub fn surface_root(&self) -> Option<PathBuf> {
		let root = self.root.file_name().map(PathBuf::from)?;
		let result = root.join(&self.relative);
		Some(result)
	}
}

impl PartialEq for Source {
	fn eq(&self, other: &Source) -> bool {
		self.relative.eq(&other.relative)
	}
}

impl Eq for Source {}

impl Hash for Source {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.relative.hash(state)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashSet;

	#[test]
	fn create_new_source() {
		let actual = Source::from_parent("/foo/bar", "/foo/bar/assets/minecraft/functions/hello.mcfunction");
		let expect = Source::test("assets/minecraft/functions/hello.mcfunction");
		assert_eq!(actual, Ok(expect));
	}

	#[test]
	#[should_panic]
	fn empty_source() {
		Source::from_parent("/foo", "").unwrap();
	}

	#[test]
	fn direct_source() {
		let actual = Source::from_parent("", "assets/minecraft/tags/functions/tick.json");
		let expect = Source::test("assets/minecraft/tags/functions/tick.json");
		assert_eq!(actual, Ok(expect));
	}

	#[test]
	fn hashing_source() {
		let mut map = HashSet::new();
		map.insert(Source::test("assets/minecraft"));

		assert!(map.contains(&Source::test("assets/minecraft")));
	}


	#[test]
	#[should_panic]
	fn hashing_invalid_source() {
		let mut map = HashSet::new();
		map.insert(Source::test("assets/minecraft"));

		assert!(map.contains(&Source::test("assets/minecraft/tags")));
	}
}