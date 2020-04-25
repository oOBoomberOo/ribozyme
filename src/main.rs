use std::path::PathBuf;
use structopt::StructOpt;
use anyhow::Result;

mod app;
mod merger;
mod rp;

use app::App;

fn main() {
	if let Err(e) = run() {
		println!("{}", e);
	}
}

fn run() -> Result<()> {
	let opt: Opt = Opt::from_args();

	let app = App::new(opt);
	app.run()
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Catalyst for merging resourcepack")]
pub struct Opt {
	#[structopt(parse(from_str))]
	directory: PathBuf,

	#[structopt(parse(from_str))]
	output: PathBuf,

	#[structopt(short, long)]
	pretty: bool
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
	pretty: bool
}