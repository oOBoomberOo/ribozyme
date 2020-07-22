use log::*;
use std::{
	collections::HashSet,
	path::{Path, PathBuf},
};
use superfusion::prelude::{Index, IndexList, Pid, Project};
use walkdir::WalkDir;

pub struct Resourcepack {
	indexes: HashSet<Index>,
	root: PathBuf,
	pid: Pid,
}

impl Resourcepack {
	pub fn from_path<P: Into<PathBuf>>(root: P, pid: Pid) -> Self {
		let root = root.into();

		info!("Initializing resourcepack from path: {}", root.display());

		let indexes = WalkDir::new(&root)
			.into_iter()
			.filter_map(resourcepack_entry)
			.filter(|p| p.is_file())
			.filter_map(|path| path.strip_prefix(&root).map(|p| p.to_owned()).ok())
			.map(|path| Index::new(pid, path))
			.collect::<HashSet<_>>();

		debug!("Found {} files from this resourcepack", indexes.len());

		Self { indexes, root, pid }
	}
}

fn resourcepack_entry(entry: walkdir::Result<walkdir::DirEntry>) -> Option<PathBuf> {
	let entry = entry.map_err(|err| error!("Entry Error: {}", err)).ok()?;
	let path = entry.into_path();
	Some(path)
}

impl Project for Resourcepack {
	fn root(&self) -> &Path {
		&self.root
	}
	fn pid(&self) -> Pid {
		self.pid
	}
	fn indexes(&self) -> IndexList {
		self.indexes.iter().collect()
	}
}
