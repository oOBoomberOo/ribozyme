use std::path::{PathBuf, Path};
use std::path::StripPrefixError;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct Source {
	pub relative: PathBuf,
	pub origin: Option<PathBuf>,
}

impl Source {
	pub fn new(relative: PathBuf, origin: Option<PathBuf>) -> Source {
		Source { relative, origin }
	}

	pub fn new_virtual(relative: impl Into<PathBuf>) -> Source {
		let relative = relative.into();
		Source::new(relative, None)
	}

	pub fn new_origin(relative: impl Into<PathBuf>, origin: impl Into<PathBuf>) -> Source {
		let relative = relative.into();
		let origin = origin.into();
		Source::new(relative, Some(origin))
	}

	pub fn from_parent(parent: impl AsRef<Path>, origin: impl Into<PathBuf>) -> Result<Source, StripPrefixError> {
		let origin = origin.into();
		let relative = origin.strip_prefix(parent).map(Path::to_path_buf)?;
		let result = Source::new_origin(relative, origin);
		
		Ok(result)
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
		let actual = Source::from_parent("/foo/bar", "/foo/bar/data/minecraft/functions/hello.mcfunction");
		let expect = Source::new_virtual("data/minecraft/functions/hello.mcfunction");
		assert_eq!(actual, Ok(expect));
	}

	#[test]
	#[should_panic]
	fn empty_source() {
		Source::from_parent("/foo", "").unwrap();
	}

	#[test]
	fn direct_source() {
		let actual = Source::from_parent("", "data/minecraft/tags/functions/tick.json");
		let expect = Source::new_virtual("data/minecraft/tags/functions/tick.json");
		assert_eq!(actual, Ok(expect));
	}

	#[test]
	fn hashing_source() {
		let mut map = HashSet::new();
		map.insert(Source::new_origin("data/minecraft", "/foo/bar/data/minecraft"));

		assert!(map.contains(&Source::new_origin("data/minecraft", "/baz/data/minecraft")));
	}


	#[test]
	#[should_panic]
	fn hashing_invalid_source() {
		let mut map = HashSet::new();
		map.insert(Source::new_origin("data/minecraft", "/foo/bar/data/minecraft"));

		assert!(map.contains(&Source::new_origin("data/minecraft/tags", "/baz/data/minecraft/tags")));
	}
}