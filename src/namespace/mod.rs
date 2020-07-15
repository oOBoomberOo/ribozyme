use std::borrow::Cow;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::convert::TryFrom;
use std::fmt;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

mod parser;
mod serialization;

#[derive(Debug, PartialEq)]
pub enum Kind {
	BlockState,
	Model,
	Texture,
	Lang,
	Font,
}

impl Kind {
	pub fn extension(&self) -> &str {
		match self {
			Self::BlockState | Self::Model | Self::Lang | Self::Font => "json",
			Self::Texture => "png",
		}
	}

	pub fn cover(&self) -> &str {
		match self {
			Self::BlockState => "blockstates",
			Self::Model => "models",
			Self::Texture => "textures",
			Self::Lang => "lang",
			Self::Font => "font",
		}
	}
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Namespace {
	pub header: String,
	pub path: PathBuf,
}

impl Namespace {
	pub fn new(header: impl Into<String>, path: impl Into<PathBuf>) -> Self {
		Self {
			header: header.into(),
			path: path.into(),
		}
	}

	pub fn to_path(&self, kind: Kind) -> PathBuf {
		let cover = kind.cover();
		let ext = kind.extension();

		let header = &self.header;
		let path = self.path.with_extension(ext);

		PathBuf::from("assets").join(header).join(cover).join(path)
	}

	pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, NamespaceError> {
		Self::try_from(path.as_ref())
	}

	pub fn raw(&self) -> (&String, &Path) {
		(&self.header, &self.path)
	}
}

impl TryFrom<&str> for Namespace {
	type Error = NamespaceError;
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let (rest, result) =
			parser::parse_namespace(value).map_err(|_| NamespaceError::InvalidNamespace)?;

		if !rest.is_empty() {
			return Err(NamespaceError::InvalidNamespace);
		}

		let (header, path) = result;
		let header = header.to_string();
		let path = PathBuf::from(path);
		let result = Self { header, path };
		Ok(result)
	}
}

impl TryFrom<&Path> for Namespace {
	type Error = NamespaceError;
	fn try_from(value: &Path) -> Result<Self, Self::Error> {
		let iter = &mut value.components();

		let _asset = advance(iter)?; // Assume `assets`
		let header = advance(iter)?;
		let _cover = advance(iter)?;
		let path: PathBuf = iter.collect();
		let path = path.with_extension("");

		if path.to_string_lossy().is_empty() {
			return Err(NamespaceError::PathTooShort);
		}

		let header = header.as_os_str().to_string_lossy().to_string();
		let result = Self::new(header, path);
		Ok(result)
	}
}

fn advance<'a, I>(iter: &mut I) -> Result<Component<'a>, NamespaceError>
where
	I: Iterator<Item = Component<'a>>,
{
	iter.next().ok_or_else(|| NamespaceError::PathTooShort)
}

impl fmt::Display for Namespace {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let path = join_with_forward_slash(self.path.components());
		write!(f, "{}:{}", self.header, path)
	}
}

/// This function exists for generating a forward-slash guaranteed display path.
///
/// Standard implementation does not guarantee this.
fn join_with_forward_slash<'a, I>(components: I) -> String
where
	I: Iterator<Item = Component<'a>>,
{
	let path: Vec<Cow<str>> = components
		.map(|c| c.as_os_str())
		.map(|s| s.to_string_lossy())
		.collect();
	path.join("/")
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NamespaceError {
	#[error("Path is too short to convert to namespace")]
	PathTooShort,
	#[error("Invalid namespace")]
	InvalidNamespace,
}

impl PartialOrd for Namespace {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.raw().partial_cmp(&other.raw())
	}
}

impl Ord for Namespace {
	fn cmp(&self, other: &Self) -> Ordering {
		self.raw().cmp(&other.raw())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn namespace_1() {
		let result = Namespace::try_from("boomber:megumin").unwrap();
		let expect = Namespace::new("boomber", "megumin");
		assert_eq!(result, expect);
	}

	#[test]
	fn namespace_2() {
		let result = Namespace::try_from("block/grass_block").unwrap();
		let expect = Namespace::new("minecraft", "block/grass_block");
		assert_eq!(result, expect);
	}

	#[test]
	fn namespace_3() {
		let result = Namespace::try_from("korone:with/slash/in/the/path").unwrap();
		let expect = Namespace::new("korone", "with/slash/in/the/path");
		assert_eq!(result, expect);
	}

	#[test]
	#[should_panic]
	fn namespace_4() {
		Namespace::try_from("").unwrap();
	}

	#[test]
	#[should_panic]
	fn namespace_5() {
		Namespace::try_from("with:too:many:colon").unwrap();
	}

	#[test]
	#[should_panic]
	fn namespace_6() {
		Namespace::try_from("InvalidCharacter").unwrap();
	}

	#[test]
	#[should_panic]
	fn namespace_7() {
		Namespace::try_from("with space: in the name").unwrap();
	}

	#[test]
	fn namespace_8() {
		let result = Namespace::try_from("bar:code").unwrap();
		let expect = Namespace::new("bar", "code");
		assert_eq!(result, expect);
	}

	#[test]
	fn namespace_9() {
		let result = Namespace::try_from("minecraft:zombie").unwrap();
		let expect = Namespace::new("minecraft", "zombie");
		assert_eq!(result, expect);
	}

	#[test]
	fn namespace_10() {
		let result = Namespace::try_from("diamond").unwrap();
		let expect = Namespace::new("minecraft", "diamond");
		assert_eq!(result, expect);
	}

	#[test]
	#[should_panic]
	fn namespace_11() {
		Namespace::try_from("foo/bar:coal").unwrap();
	}

	#[test]
	fn namespace_12() {
		let result = Namespace::try_from("minecraft/villager").unwrap();
		let expect = Namespace::new("minecraft", "minecraft/villager");
		assert_eq!(result, expect);
	}

	#[test]
	fn namespace_13() {
		let result = Namespace::try_from("mypack_recipe").unwrap();
		let expect = Namespace::new("minecraft", "mypack_recipe");
		assert_eq!(result, expect);
	}

	#[test]
	#[should_panic]
	fn namespace_14() {
		Namespace::try_from("mymap:schrödingers_var").unwrap();
	}

	#[test]
	#[should_panic]
	fn namespace_15() {
		Namespace::try_from("custom_pack:Capital").unwrap();
	}

	#[test]
	fn namespace_16() {
		let result =
			Namespace::from_path("assets/minecraft/models/item/diamond_sword.json").unwrap();
		let expect = Namespace::new("minecraft", "item/diamond_sword");
		assert_eq!(result, expect);
	}

	#[test]
	fn namespace_17() {
		let result = Namespace::from_path("assets/minecraft/").unwrap_err();
		let expect = NamespaceError::PathTooShort;
		assert_eq!(result, expect);
	}

	#[test]
	fn namespace_18() {
		let result = Namespace::from_path("assets/minecraft/models").unwrap_err();
		let expect = NamespaceError::PathTooShort;
		assert_eq!(result, expect);
	}

	#[test]
	fn namespace_19() {
		let result = Namespace::from_path("assets/boomber/models/megumin").unwrap();
		let expect = Namespace::new("boomber", "megumin");
		assert_eq!(result, expect);
	}

	#[test]
	fn namespace_20() {
		let result = Namespace::from_path("assets/boomber/textures/megumin").unwrap();
		let expect = Namespace::new("boomber", "megumin");
		assert_eq!(result, expect);
	}
}
