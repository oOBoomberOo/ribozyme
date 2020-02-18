use std::path::PathBuf;
use std::ffi::OsString;
#[derive(Default, Eq, Clone)]
pub struct Resource {
	location: PathBuf,
	name: OsString,
	content: Content,
	format: ResourceFormat
}

use crate::{ProgramResult, ProgramError};
use super::template::{Model, Lang};
use std::fs;
use std::io;
use std::io::Write;
use std::iter::FromIterator;
use std::ffi::OsStr;
use zip::ZipWriter;
use zip::write::FileOptions;
use serde_json as js;
impl Resource {
	pub fn from_entry(entry: io::Result<fs::DirEntry>, format: ResourceType, parent: impl Into<PathBuf>) -> ProgramResult<Resource> {
		let entry: fs::DirEntry = entry?;
		let parent = parent.into();
		let name = entry.file_name();
		let path = entry.path();
		let location = path.strip_prefix(&parent)?.to_path_buf();

		let file_type = match entry.file_type() {
			Ok(result) => result,
			Err(error) => return Err(ProgramError::IoWithPath(path, error))
		};

		if file_type.is_dir() {
			let child_iter = match path.read_dir() {
				Ok(entries) => entries.filter_map(|entry| Resource::from_entry(entry, format, &parent).ok()),
				Err(error) => return Err(ProgramError::IoWithPath(path, error))
			};

			let child: HashSet<Resource> = HashSet::from_iter(child_iter);
			let content = Content::Folder(child);
			let format = ResourceFormat::Folder(format);

			let result = Resource { location, name, content, format };

			Ok(result)
		}
		else {
			let content = match fs::read(&path) {
				Ok(result) => result,
				Err(error) => return Err(ProgramError::IoWithPath(path, error))
			};
			let content = Content::File(content);
			let format = ResourceFormat::File(format);

			if path.file_name() == Some(OsStr::new(".DS_Store")) {
				return Err(ProgramError::InvalidResourceFormat(path, format))
			}
			
			let result = Resource { location, name, content, format };
			Ok(result)
		}
	}

	pub fn from_namespace(entry: io::Result<fs::DirEntry>, parent: impl Into<PathBuf>) -> ProgramResult<Resource> {
		let entry: fs::DirEntry = entry?;
		let parent = parent.into();

		let format = match entry.file_name().to_str() {
			Some("models") => ResourceType::Model,
			Some("lang") => ResourceType::Lang,
			_ => ResourceType::Other
		};

		Resource::from_entry(Ok(entry), format, parent)
	}

	pub fn merge(self, other: Resource) -> ResourceResult<Resource> {
		match self.format {
			ResourceFormat::File(kind) => match kind {
				ResourceType::Model => {
					let content = self.get_content()?;
					let other_content = other.get_content()?;
					let location = other.location.clone();

					let model: Model = match js::from_slice(content) {
						Ok(result) => result,
						Err(error) => return Err(ResourceError::Serde(location, error))
					};
					let other_model: Model = match js::from_slice(other_content) {
						Ok(result) => result,
						Err(error) => return Err(ResourceError::Serde(location, error))
					};

					let merged_model = model.merge(other_model);
					let content = match js::to_vec(&merged_model) {
						Ok(result) => result,
						Err(error) => return Err(ResourceError::Serde(location, error))
					};

					let content = Content::File(content);
					let name = other.name;
					let format = other.format;

					let result = Resource { location, name, content, format };

					Ok(result)
				},
				ResourceType::Lang => {
					let content = self.get_content()?;
					let other_content = other.get_content()?;
					let location = other.location.clone();

					let mut content: Lang = match js::from_slice(content) {
						Ok(result) => result,
						Err(error) => return Err(ResourceError::Serde(location, error))
					};
					let other_content: Lang = match js::from_slice(other_content) {
						Ok(result) => result,
						Err(error) => return Err(ResourceError::Serde(location, error))
					};

					content.extend(other_content.into_iter());
					let content_str = match js::to_vec_pretty(&content) {
						Ok(result) => result,
						Err(error) => return Err(ResourceError::Serde(location, error))
					};

					let name = other.name;
					let content = Content::File(content_str);
					let format = other.format;

					let result = Resource { location, name, content, format };

					Ok(result)
				}
				ResourceType::Other => Ok(other)
			},
			ResourceFormat::Folder(_kind) => {
				let mut content = self.get_child()?;
				let other_content = other.get_child()?;

				for resource in other_content {
					let result = match content.take(&resource) {
						Some(original) => original.merge(resource)?,
						None => resource
					};

					content.insert(result);
				}

				let location = other.location;
				let name = other.name;
				let content = Content::Folder(content);
				let format = other.format;

				let result = Resource { location, name, content, format };
				
				Ok(result)
			},
			ResourceFormat::Unknown => {
				Ok(other)
			}
		}
	}

	fn get_content(&self) -> ResourceResult<&[u8]> {
		match &self.content {
			Content::File(contents) => Ok(contents),
			Content::Folder(_) => Err(ResourceError::FileAsDirectory(self.location.clone()))
		}
	}

	fn get_child(&self) -> ResourceResult<HashSet<Resource>> {
		match &self.content {
			Content::Folder(contents) => Ok(contents.clone()),
			Content::File(_) => Err(ResourceError::DirectoryAsFile(self.location.clone()))
		}
	}

	pub fn build(self, zip: &mut ZipWriter<fs::File>, options: FileOptions) -> ProgramResult<()> {
		match self.content {
			Content::File(data) => {
				zip.start_file_from_path(&self.location, options)?;
				zip.write_all(&data)?;

				Ok(())
			},
			Content::Folder(child) => {
				for resource in child {
					resource.build(zip, options)?;
				}

				Ok(())
			}
		}
	}
}

impl fmt::Debug for Resource {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.content {
			Content::File(_) => write!(f, "{:?}", self.name),
			Content::Folder(child) => write!(f, "{:?}: {:#?}", self.name, child)
		}
	}
}

use std::hash::{Hash, Hasher};
impl Hash for Resource {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name.hash(state);
		self.format.hash(state);
	}
}
impl PartialEq for Resource {
	fn eq(&self, other: &Resource) -> bool {
		self.name == other.name && self.format == other.format
	}
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ResourceFormat {
	File(ResourceType),
	Folder(ResourceType),
	Unknown
}

impl Default for ResourceFormat {
	fn default() -> ResourceFormat {
		ResourceFormat::Unknown
	}
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum ResourceType {
	Other,
	Model,
	Lang
}

impl Default for ResourceType {
	fn default() -> ResourceType {
		ResourceType::Other
	}
}

use std::collections::HashSet;
#[derive(PartialEq, Eq, Clone)]
pub enum Content {
	File(Vec<u8>),
	Folder(HashSet<Resource>)
}

impl Default for Content {
	fn default() -> Content {
		Content::File(Vec::default())
	}
}

impl fmt::Debug for Content {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Content::File(data) => write!(f, "{:?}", String::from_utf8(data.clone())),
			Content::Folder(child) => write!(f, "{:#?}", child)
		}
	}
}

type ResourceResult<T> = Result<T, ResourceError>;

#[derive(Debug)]
pub enum ResourceError {
	FileAsDirectory(PathBuf),
	DirectoryAsFile(PathBuf),
	Serde(PathBuf, serde_json::Error)
}

use console::style;
use std::fmt;
impl fmt::Display for ResourceError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ResourceError::FileAsDirectory(path) => write!(f, "'{}' is recognized as directory even though it's a file. {}", style(path.display()).cyan(), style("(this is a logic error please contact @Boomber)").red().underlined()),
			ResourceError::DirectoryAsFile(path) => write!(f, "'{}' is recognized as file even though it's a directory. {}", style(path.display()).cyan(), style("(this is a logic error please contact @Boomber)").red().underlined()),
			ResourceError::Serde(path, error) => write!(f, "[{}] {}", style(path.display()).cyan(), error)
		}
	}
}