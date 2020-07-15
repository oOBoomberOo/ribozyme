use super::workspace;
use super::Error as Err;
use anyhow::Result;
use std::path::Path;
use superfusion::prelude::{Error, File, Index, Pid, Relation};
use log::*;

mod blockstate;
mod lang;
mod model;
mod other;
mod texture;

pub use lang::Lang;
pub use model::Model;
pub use other::Other;
pub use texture::Texture;

pub enum Asset {
	Lang(Box<Lang>),
	Texture(Box<Texture>),
	Model(Box<Model>),
	Other(Box<Other>),
}

impl Asset {
	pub fn new(path: &Path, pid: Pid) -> Result<Self> {
		let asset = {
			if workspace::models_folder(path) {
				info!("Create model for {}", path.display());
				let data = Model::new(path, pid)?;
				Asset::Model(Box::new(data))
			} else if workspace::lang_folder(path) {
				info!("Create lang file for {}", path.display());
				let data = Lang::new(path)?;
				Asset::Lang(Box::new(data))
			} else if workspace::texture_folder(path) {
				info!("Create texture file for {}", path.display());
				let data = Texture::new(path)?;
				Asset::Texture(Box::new(data))
			} else {
				info!("Create other file for {}", path.display());
				let data = Other::new(path)?;
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
			Other(other) => other.relation(),
		}
	}
	fn data(self) -> Vec<u8> {
		use Asset::*;

		match self {
			Lang(lang) => lang.data(),
			Texture(texture) => texture.data(),
			Model(model) => model.data(),
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
			(Other(a), Other(b)) => Other(Box::new(a.merge(*b)?)),
			_ => return Err(Error::custom(Err::IncompatibleFile)),
		};
		Ok(result)
	}
}
