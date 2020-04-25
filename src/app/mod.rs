use crate::{Opt, Style};
use crate::merger::{Merger};
use anyhow::Result;
use std::fs;
use rayon::prelude::*;

pub struct App {
	opt: Opt,
}

impl App {
	pub fn new(opt: Opt) -> App {
		App { opt }
	}

	pub fn run(&self) -> Result<()> {
		let directory = &self.opt.directory;
		let style = Style { pretty: self.opt.pretty };

		let merger = Merger::from_path(directory)?;
		let conflicts = merger.into_conflict_solver().solve(style)?;

		let output = &self.opt.output;
		fs::remove_dir_all(&output)?;
		conflicts
			.par_iter()
			.map(|(relative, file)| (output.join(relative), file))
			.try_for_each(|(path, file)| file.write(path))?;
		
		Ok(())
	}
}
