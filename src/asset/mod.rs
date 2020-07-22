use super::workspace;
use super::Error as Err;
use anyhow::{Context, Result};
use std::path::Path;
use superfusion::prelude::{Error, File, Index, Pid, Relation};
use log::*;

mod blockstate;
mod lang;
mod model;
mod other;
mod texture;

use crate::namespace::{Kind, Namespace};
pub use blockstate::BlockState;
pub use lang::Lang;
pub use model::Model;
pub use other::Other;
pub use texture::Texture;

pub fn into_index(kind: Kind, namespace: &Namespace, pid: Pid) -> Index {
	let path = namespace.to_path(kind);
	Index::new(pid, path)
}

pub fn from_index(index: &Index) -> Result<Namespace> {
	let path = index.path();
	Namespace::from_path(path)
		.with_context(|| format!("'{}' cannot be converted to namespace", path.display()))
}

pub enum Asset {
	Lang(Box<Lang>),
	Texture(Box<Texture>),
	Model(Box<Model>),
	Other(Box<Other>),
	BlockState(Box<BlockState>),
}

impl Asset {
	pub fn new(path: &Path, pid: Pid) -> Result<Self> {
		debug!("Creating asset from {}", path.display());

		let asset = {
			if workspace::models_folder(path) {
				let data = Model::new(path, pid)
					.with_context(|| format!("Failed to read model file at {}", path.display()))?;
				Asset::Model(Box::new(data))
			} else if workspace::lang_folder(path) {
				let data = Lang::new(path).with_context(|| {
					format!("Failed to read language file at {}", path.display())
				})?;
				Asset::Lang(Box::new(data))
			} else if workspace::texture_folder(path) {
				let data = Texture::new(path).with_context(|| {
					format!("Failed to read texture file at {}", path.display())
				})?;
				Asset::Texture(Box::new(data))
			} else if workspace::blockstate_folder(path) {
				let data = BlockState::new(path, pid).with_context(|| {
					format!("Failed to read blockstate file at {}", path.display())
				})?;
				Asset::BlockState(Box::new(data))
			} else {
				let data = Other::new(path)
					.with_context(|| format!("Failed to read file at {}", path.display()))?;
				Asset::Other(Box::new(data))
			}
		};
		Ok(asset)
	}
}

impl File for Asset {
	fn relation(&self) -> Vec<Relation> {
		use Asset::*;

		match self {
			Lang(lang) => lang.relation(),
			Texture(texture) => texture.relation(),
			Model(model) => model.relation(),
			BlockState(blockstate) => blockstate.relation(),
			Other(other) => other.relation(),
		}
	}
	fn data(self) -> Vec<u8> {
		use Asset::*;

		match self {
			Lang(lang) => lang.data(),
			Texture(texture) => texture.data(),
			Model(model) => model.data(),
			BlockState(blockstate) => blockstate.data(),
			Other(other) => other.data(),
		}
	}
	fn modify_relation(self, from: &Index, to: &Index) -> Self
	where
		Self: Sized,
	{
		use Asset::*;

		match self {
			Lang(lang) => Lang(Box::new(lang.modify_relation(from, to))),
			Texture(texture) => Texture(Box::new(texture.modify_relation(from, to))),
			Model(model) => Model(Box::new(model.modify_relation(from, to))),
			BlockState(blockstate) => BlockState(Box::new(blockstate.modify_relation(from, to))),
			Other(other) => Other(Box::new(other.modify_relation(from, to))),
		}
	}

	fn merge(self, other: Self) -> Result<Self, Error>
	where
		Self: Sized,
	{
		use Asset::*;

		let result = match (self, other) {
			(Lang(a), Lang(b)) => Lang(Box::new(a.merge(*b)?)),
			(Texture(a), Texture(b)) => Texture(Box::new(a.merge(*b)?)),
			(Model(a), Model(b)) => Model(Box::new(a.merge(*b)?)),
			(BlockState(a), BlockState(b)) => BlockState(Box::new(a.merge(*b)?)),
			(Other(a), Other(b)) => Other(Box::new(a.merge(*b)?)),
			_ => return Err(Error::custom(Err::IncompatibleFile)),
		};
		Ok(result)
	}
}
