use super::Source;
use crate::merger::Conflict;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Component, PathBuf};
use std::fs;
use anyhow::{Result};

#[derive(Debug)]
pub struct Resource {
	source: Source,
}

impl Resource {
	pub fn new(source: Source) -> Resource {
		Resource { source }
	}

	pub fn relative_path(&self) -> PathBuf {
		self.source.relative.to_owned()
	}

	pub fn into_conflict(self, conflicts: &mut HashMap<PathBuf, Conflict>) {
		let key = self.relative_path();
		if let Some(conflict) = conflicts.get_mut(&key) {
			conflict.add(self);
		} else {
			conflicts.insert(key, Conflict::with_resource(self));
		}
	}

	pub fn kind(&self) -> ResourceKind {
		let path = &self.source.relative;
		let mut components = path.components().skip(2); // Skip `data` folder and namespace folder

		components
			.next()
			.map_or(ResourceKind::Other, Resource::component_to_kind)
	}

	pub fn component_to_kind(kind: Component) -> ResourceKind {
		match kind.as_os_str().to_str() {
			Some("models") => ResourceKind::Model,
			Some("textures") => ResourceKind::Texture,
			Some("lang") => ResourceKind::Lang,
			_ => ResourceKind::Other,
		}
	}

	pub fn data(&self) -> Result<Vec<u8>> {
		let path = self.origin();
		let result = fs::read(path)?;
		Ok(result)
	}

	pub fn origin(&self) -> &PathBuf {
		&self.source.origin
	}
}

impl Hash for Resource {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.source.hash(state)
	}
}

impl PartialEq for Resource {
	fn eq(&self, other: &Resource) -> bool {
		self.source.eq(&other.source)
	}
}

impl Eq for Resource {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
	Model,
	Texture,
	Lang,
	Other,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn convert_to_conflict() {
		let source = Source::new_virtual("test.mcfunction");
		let resource = Resource::new(source.clone());

		let mut actual = HashMap::new();
		resource.into_conflict(&mut actual);

		let mut expect = HashMap::new();
		let expected_conflicts = Conflict::new(vec![Resource::new(source)]);
		expect.insert(PathBuf::from("test.mcfunction"), expected_conflicts);

		assert_eq!(actual, expect);
	}

	#[test]
	fn push_to_conflicts() {
		let mut actual = HashMap::new();
		Resource::new(Source::new_origin("test.mcfunction", "foo/test.mcfunction"))
			.into_conflict(&mut actual);
		Resource::new(Source::new_origin("test.mcfunction", "bar/test.mcfunction"))
			.into_conflict(&mut actual);

		let mut expect = HashMap::new();
		let expected_conflicts = Conflict::new(vec![
			Resource::new(Source::new_origin("test.mcfunction", "foo/test.mcfunction")),
			Resource::new(Source::new_origin("test.mcfunction", "bar/test.mcfunction")),
		]);
		expect.insert(PathBuf::from("test.mcfunction"), expected_conflicts);

		assert_eq!(actual, expect);
	}

	#[test]
	fn model_resource() {
		let resource = Resource::new(Source::new_virtual("assets/boomber/models/test/hello.json"));
		assert_eq!(resource.kind(), ResourceKind::Model);
	}

	#[test]
	fn texture_resource() {
		let resource = Resource::new(Source::new_virtual("assets/boomber/textures/item/test.png"));
		assert_eq!(resource.kind(), ResourceKind::Texture);
	}

	#[test]
	fn lang_resource() {
		let resource = Resource::new(Source::new_virtual("assets/minecraft/lang/en_us.png"));
		assert_eq!(resource.kind(), ResourceKind::Lang);
	}

	#[test]
	fn invalid_resource() {
		let resource = Resource::new(Source::new_virtual("assets/boomber"));
		assert_eq!(resource.kind(), ResourceKind::Other);
	}

	#[test]
	fn convert_component_to_kind() {
		let models = Resource::component_to_kind(Component::Normal("models".as_ref()));
		assert_eq!(models, ResourceKind::Model);

		let textures = Resource::component_to_kind(Component::Normal("textures".as_ref()));
		assert_eq!(textures, ResourceKind::Texture);

		let lang = Resource::component_to_kind(Component::Normal("lang".as_ref()));
		assert_eq!(lang, ResourceKind::Lang);

		let nonsense = Resource::component_to_kind(Component::Normal("hello".as_ref()));
		assert_eq!(nonsense, ResourceKind::Other);

		let another_nonsense = Resource::component_to_kind(Component::RootDir);
		assert_eq!(another_nonsense, ResourceKind::Other);

		let even_more_nonsense = Resource::component_to_kind(Component::CurDir);
		assert_eq!(even_more_nonsense, ResourceKind::Other);

		let the_most_nonsense_of_all = Resource::component_to_kind(Component::ParentDir);
		assert_eq!(the_most_nonsense_of_all, ResourceKind::Other);
	}
}
