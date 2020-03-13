use std::ffi::OsString;
use std::path::PathBuf;
#[derive(Default, Eq, Clone)]
pub struct Resource {
	location: ResourcePath,
	relative: PathBuf,
	name: OsString,
	content: Content,
	format: ResourceFormat,
}

use super::template::{Lang, Model};
use crate::{ProgramError, ProgramResult, ResourcePath};
use crate::utils::bom_fix_vec;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json as js;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io;
use std::iter::FromIterator;
use tar::Builder;
use flate2::write::GzEncoder;
use indicatif::ProgressBar;
impl Resource {
	pub fn from_entry(
		entry: fs::DirEntry,
		format: ResourceType,
		root: &ResourcePath,
	) -> ProgramResult<Resource> {
		let name = entry.file_name();
		let path = entry.path();
		let relative = path.strip_prefix(&root.physical)?.to_path_buf();
		let location = root.join(&relative);

		let file_type = match entry.file_type() {
			Ok(result) => result,
			Err(error) => return Err(ProgramError::IoWithPath(path, error)),
		};

		if file_type.is_dir() {
			let child_iter = match path.read_dir() {
				Ok(entries) => entries,
				Err(error) => return Err(ProgramError::IoWithPath(path, error)),
			};

			let child: Result<Vec<Resource>, _> = child_iter
				.par_bridge()
				.map(|entry| Resource::from_entry(entry?, format, root))
				.filter_map(|resource| match resource {
					Err(ProgramError::Resource(ResourceError::IgnoredFile)) => None,
					other => Some(other),
				})
				.collect();

			let child: HashSet<Resource> = HashSet::from_iter(child?);
			let content = Content::Folder(child);
			let format = ResourceFormat::Folder(format);

			let result = Resource {
				location,
				relative,
				name,
				content,
				format,
			};

			Ok(result)
		} else {
			if let Some(extension) = path.extension() {
				if format == ResourceType::Model && extension != "json" {
					return Err(ResourceError::IgnoredFile.into());
				}
				if format == ResourceType::Lang && extension != "json" {
					return Err(ResourceError::IgnoredFile.into());
				}
			}

			let mut content = match fs::read(&path) {
				Ok(result) => result,
				Err(error) => return Err(ProgramError::IoWithPath(path, error)),
			};

			if content[0] == 239 {
				content.remove(0);
			}

			let content = Content::File(content.to_vec());
			let format = ResourceFormat::File(format);

			if path.file_name() == Some(OsStr::new(".DS_Store")) {
				return Err(ResourceError::IgnoredFile.into());
			}

			let result = Resource {
				location,
				relative,
				name,
				content,
				format,
			};
			Ok(result)
		}
	}

	pub fn from_namespace(
		entry: io::Result<fs::DirEntry>,
		root: &ResourcePath,
	) -> ProgramResult<Resource> {
		let entry = entry?;
		let format = match &entry.file_name().to_str() {
			Some("models") => ResourceType::Model,
			Some("lang") => ResourceType::Lang,
			_ => ResourceType::Other,
		};

		Resource::from_entry(entry, format, root)
	}

	pub fn merge(self, other: Resource, progress_bar: &ProgressBar) -> ResourceResult<Resource> {
		match self.format {
			ResourceFormat::File(kind) => match kind {
				ResourceType::Model => {
					let model: Model = self.get_content()?;
					let other_model: Model = other.get_content()?;

					let merged_model = model.merge(other_model);
					let content = match js::to_vec(&merged_model) {
						Ok(result) => result,
						Err(error) => {
							return Err(ResourceError::Serde(other.location.origin, error))
						}
					};

					let location = other.location;
					let content = Content::File(content);
					let name = other.name;
					let format = other.format;
					let relative = other.relative;

					let result = Resource {
						location,
						relative,
						name,
						content,
						format,
					};

					Ok(result)
				}
				ResourceType::Lang => {
					let mut content: Lang = self.get_content()?;
					let other_content: Lang = other.get_content()?;

					content.extend(other_content.into_iter());
					let content_str = match js::to_vec_pretty(&content) {
						Ok(result) => result,
						Err(error) => {
							return Err(ResourceError::Serde(other.location.origin, error))
						}
					};

					let name = other.name;
					let content = Content::File(content_str);
					let format = other.format;
					let location = other.location;
					let relative = other.relative;

					let result = Resource {
						location,
						relative,
						name,
						content,
						format,
					};

					Ok(result)
				}
				ResourceType::Other => Ok(other),
			},
			ResourceFormat::Folder(_kind) => {
				let mut content = self.get_child()?;

				for resource in other.get_child()? {
					let resource = match content.take(&resource) {
						None => Ok(resource),
						Some(original) => original.merge(resource, progress_bar),
					};

					content.replace(resource?);
				}

				progress_bar.inc(content.len() as u64);

				let location = other.location;
				let relative = other.relative;
				let name = other.name;
				let content = Content::Folder(content);
				let format = other.format;

				let result = Resource {
					location,
					relative,
					name,
					content,
					format,
				};

				Ok(result)
			}
			ResourceFormat::Unknown => Ok(other),
		}
	}

	fn get_content<'a, T: Deserialize<'a> + Serialize>(&'a self) -> ResourceResult<T> {
		let content = match &self.content {
			Content::File(contents) => contents,
			Content::Folder(_) => {
				return Err(ResourceError::FileAsDirectory(
					self.location.origin.to_owned(),
				))
			}
		};

		let content = bom_fix_vec(content);
		match js::from_slice(content) {
			Ok(result) => Ok(result),
			Err(error) => Err(ResourceError::Serde(self.location.origin.to_owned(), error)),
		}
	}

	fn get_child(&self) -> ResourceResult<HashSet<Resource>> {
		match &self.content {
			Content::Folder(contents) => Ok(contents.clone()),
			Content::File(_) => Err(ResourceError::DirectoryAsFile(self.location.origin.clone())),
		}
	}

	pub fn build(self, archive: &mut Builder<GzEncoder<File>>, progress_bar: &ProgressBar) -> ProgramResult<()> {
		match self.content {
			Content::File(_) => {
				let mut file = File::open(&self.location.physical)?;
				progress_bar.inc(1);
				if let Err(error) = archive.append_file(self.relative, &mut file) {
					return Err(ProgramError::IoWithPath(self.location.physical, error));
				}
				Ok(())
			}
			Content::Folder(child) => {
				if let Err(error) = archive.append_dir(self.relative, &self.location.physical) {
					return Err(ProgramError::IoWithPath(self.location.physical, error));
				}

				child.into_iter().try_for_each(|x| x.build(archive, progress_bar))
			}
		}
	}

	pub fn count(&self) -> u64 {
		match &self.content {
			Content::File(_) => 1,
			Content::Folder(child) => child.iter().fold(0, |acc, resource| acc + resource.count())
		}
	}
}

impl fmt::Debug for Resource {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.content {
			Content::File(_) => write!(f, "{:?}", self.name),
			Content::Folder(child) => write!(f, "{:?} [{}]: {:#?}", self.name, child.len(), child),
		}
	}
}

use std::hash::{Hash, Hasher};
impl Hash for Resource {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name.hash(state);
	}
}
impl PartialEq for Resource {
	fn eq(&self, other: &Resource) -> bool {
		self.name == other.name
	}
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ResourceFormat {
	File(ResourceType),
	Folder(ResourceType),
	Unknown,
}

impl fmt::Display for ResourceFormat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ResourceFormat::File(_) => write!(f, "File"),
			ResourceFormat::Folder(_) => write!(f, "Folder"),
			ResourceFormat::Unknown => write!(f, "Unknown"),
		}
	}
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
	Lang,
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
	Folder(HashSet<Resource>),
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
			Content::Folder(child) => write!(f, "{:#?} ({})", child, child.len()),
		}
	}
}

type ResourceResult<T> = Result<T, ResourceError>;

#[derive(Debug)]
pub enum ResourceError {
	FileAsDirectory(PathBuf),
	DirectoryAsFile(PathBuf),
	Serde(PathBuf, serde_json::Error),
	IgnoredFile,
}

use colored::*;
use std::fmt;
impl fmt::Display for ResourceError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ResourceError::FileAsDirectory(path) => write!(
				f,
				"'{}' is recognized as directory even though it's a file. {}",
				path.display().to_string().cyan(),
				"(this is a logic error please contact @Boomber)"
					.red()
					.underline()
			),
			ResourceError::DirectoryAsFile(path) => write!(
				f,
				"'{}' is recognized as file even though it's a directory. {}",
				path.display().to_string().cyan(),
				"(this is a logic error please contact @Boomber)"
					.red()
					.underline()
			),
			ResourceError::Serde(path, error) => {
				write!(f, "[{}] {}", path.display().to_string().cyan(), error)
			}
			ResourceError::IgnoredFile => write!(f, "Ignored File"),
		}
	}
}
