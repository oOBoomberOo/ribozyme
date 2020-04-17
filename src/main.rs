use std::path::PathBuf;
use structopt::StructOpt;
use anyhow::Result;

mod app;
mod merger;
mod rp;

use merger::Merger;

fn main() {
	if let Err(e) = run() {
		println!("{}", e);
	}
}

fn run() -> Result<()> {
	let opt: Opt = Opt::from_args();
	let merger = Merger::from_path(opt.directory)?;
	let conflicts = merger.get_conflict();

	let key = PathBuf::from("assets/minecraft/models/item/carrot_on_a_stick.json");
	println!("{:#?}", conflicts.get(&key));
	Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Catalyst for merging resourcepack")]
struct Opt {
	#[structopt(parse(from_str))]
	directory: PathBuf,
}