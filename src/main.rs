use std::path::PathBuf;
use structopt::StructOpt;
use anyhow::Result;

mod app;

fn main() {
	if let Err(e) = run() {
		println!("{}", e);
	}
}

fn run() -> Result<()> {
	todo!()
}

#[derive(Debug, StructOpt)]
#[structopt()]
struct Opt {
	#[structopt(parse(from_str))]
	directory: PathBuf,
}