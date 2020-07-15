use anyhow::Result;
use log::*;
use std::path::PathBuf;
use structopt::StructOpt;
use superfusion::prelude::{Index, Workspace as _};
use std::io::Write;

mod asset;
mod error;
mod namespace;
mod resourcepack;
mod workspace;

use asset::Asset;
use error::Error;
use resourcepack::Resourcepack;
use workspace::Workspace;

fn main() {
	let mut builder = env_logger::builder();
	builder
		.format(|buf, rec| writeln!(buf, "[{}] {}", rec.level(), rec.args()))
		.init();

	if let Err(e) = run() {
		println!("{}", e);
	}
}

fn run() -> Result<()> {
	let opt: Opt = Opt::from_args();

	if opt.output.exists() {
		std::fs::remove_dir_all(&opt.output)?;
	}

	let mut logger = Logger;
	let workspace = Workspace::from_path(&opt.directory)?;
	let timeline = workspace.resolve(&mut logger);
	timeline.export_to(&opt.output)?;

	Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Catalyst for merging resourcepack")]
pub struct Opt {
	#[structopt(parse(from_str))]
	directory: PathBuf,

	#[structopt(parse(from_str))]
	output: PathBuf,
}

struct Logger;

impl superfusion::logger::Logger for Logger {
	fn add(&mut self, index: &Index) {
		debug!("[+] {}", index);
	}
	fn replace(&mut self, conflict: &Index, with: &Index) {
		debug!("[!] {} → {}", conflict, with);
	}
	fn merge(&mut self, conflict: &Index, with: &Index) {
		debug!("[$] {} → {}", conflict, with);
	}
	fn rename(&mut self, conflict: &Index, index: &Index) {
		debug!("[%] {} → {}", conflict, index);
	}
}
