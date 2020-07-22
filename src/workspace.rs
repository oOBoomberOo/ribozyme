use super::{Asset, Resourcepack};
use anyhow::Result;
use glob::Pattern;
use lazy_static::lazy_static;
use log::*;
use std::{
	fs::DirEntry,
	io,
	path::{Path, PathBuf},
};
use superfusion::criteria::Composite;
use superfusion::prelude::{Index, Pid, Strategy};

fn criteria() -> Composite {
	Composite::new()
		.with(|path| path.join("pack.mcmeta").is_file())
		.with(|path| path.join("assets").is_dir())
}

pub struct Workspace {
	projects: Vec<Resourcepack>,
}

impl Workspace {
	pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
		let composite = criteria();

		let path = path.as_ref();

		info!("Initialize Workspace from path: {}", path.display());

		let projects = path
			.read_dir()?
			.filter_map(path_entry)
			.filter(|path| composite.check(path))
			.enumerate()
			.map(resourcepack)
			.collect::<Vec<_>>();

		debug!("Found {} project(s) in total", projects.len());

		let result = Self { projects };
		Ok(result)
	}
}

fn path_entry(entry: io::Result<DirEntry>) -> Option<PathBuf> {
	let entry = entry.ok()?;
	let path = entry.path();
	Some(path)
}

fn resourcepack((n, path): (usize, PathBuf)) -> Resourcepack {
	let pid = Pid::new(n);
	Resourcepack::from_path(path, pid)
}

impl superfusion::prelude::Workspace for Workspace {
	type Project = Resourcepack;
	type File = Asset;
	fn projects(&self) -> &[Self::Project] {
		&self.projects
	}
	fn strategy(&self, index: &Index) -> Strategy {
		let path = index.path();

		let is_vanilla = minecraft_folder(path);
		let is_models = models_folder(path);
		let is_lang = lang_folder(path);
		let is_texture = texture_folder(path);
		let is_pack_meta = pack_meta(path);

		if is_vanilla && is_models {
			return Strategy::Merge;
		}

		if !is_vanilla && (is_models || is_texture) {
			return Strategy::Rename;
		}

		if is_lang || is_pack_meta {
			return Strategy::Merge;
		}

		Strategy::Replace
	}
	fn file(path: &Path, pid: Pid) -> Option<Self::File> {
		Asset::new(path, pid)
			.map_err(|err| error!("\n{:?}", err))
			.ok()
	}
}

lazy_static! {
	static ref MINECRAFT_FOLDER: Pattern = Pattern::new("**/assets/minecraft/**").unwrap();
	static ref MODEL: Pattern = Pattern::new("**/assets/*/models/**/*.json").unwrap();
	static ref LANG: Pattern = Pattern::new("**/assets/*/lang/**/*.json").unwrap();
	static ref TEXTURE: Pattern = Pattern::new("**/assets/*/textures/**/*.png").unwrap();
	static ref PACK_META: Pattern = Pattern::new("**/pack.mcmeta").unwrap();
	static ref BLOCKSTATE: Pattern = Pattern::new("**/assets/*/blockstates/**/*.json").unwrap();
}

pub fn minecraft_folder(path: &Path) -> bool {
	MINECRAFT_FOLDER.matches_path(path)
}

pub fn models_folder(path: &Path) -> bool {
	MODEL.matches_path(path)
}

pub fn lang_folder(path: &Path) -> bool {
	LANG.matches_path(path)
}

pub fn texture_folder(path: &Path) -> bool {
	TEXTURE.matches_path(path)
}

pub fn pack_meta(path: &Path) -> bool {
	PACK_META.matches_path(path)
}

pub fn blockstate_folder(path: &Path) -> bool {
	BLOCKSTATE.matches_path(path)
}
