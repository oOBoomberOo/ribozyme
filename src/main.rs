use std::path::PathBuf;
use structopt::StructOpt;
use anyhow::Result;
use std::collections::HashMap;
use rayon::prelude::*;
use std::fs;

mod app;
mod merger;
mod rp;

use merger::File;
use merger::Merger;

fn main() {
	if let Err(e) = run() {
		println!("{}", e);
	}
}

fn run() -> Result<()> {
	let opt: Opt = Opt::from_args();
	let merger = Merger::from_path(opt.directory)?;
	let conflicts: HashMap<PathBuf, File> = merger
		.into_conflict_solver()
		.solve()?;

	let output = opt.output;
	fs::remove_dir_all(&output)?;
	conflicts
		.par_iter()
		.map(|(relative, file)| (output.join(relative), file))
		.try_for_each(|(path, file)| file.write(path))?;

	Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Catalyst for merging resourcepack")]
struct Opt {
	#[structopt(parse(from_str))]
	directory: PathBuf,

	#[structopt(parse(from_str))]
	output: PathBuf
}