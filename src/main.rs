use std::path::PathBuf;
use structopt::StructOpt;
use anyhow::Result;
use std::collections::HashMap;

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
	let conflicts: HashMap<PathBuf, File> = merger.into_conflict_solver().into_iter().collect();

	println!("{:#?}", conflicts);

	Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Catalyst for merging resourcepack")]
struct Opt {
	#[structopt(parse(from_str))]
	directory: PathBuf,
}