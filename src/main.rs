use anyhow::Result;
use log::*;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Instant;
use structopt::StructOpt;
use superfusion::prelude::Workspace as _;
use tempfile::tempdir;
use zip::ZipWriter;
use zip_extensions::ZipWriterExtensions;

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
	env_logger::init();

	if let Err(e) = run() {
		println!("{}", e);
	}
}

fn run() -> Result<()> {
	let opt: Opt = Opt::from_args();

	debug!("Receive command argument: {:?}", opt);

	let time = Instant::now();

	let output = if opt.zip {
		let tempdir = tempdir()?;
		let output_dir = tempdir.path();
		debug!("Create temporary directory at {}", output_dir.display());

		merger(&opt.input, output_dir)?;

		let output = opt.output.with_extension("zip");
		zip_dir(&output, output_dir)?;

		output
	} else {
		if opt.output.exists() {
			info!("Cleaning output directory...");
			std::fs::remove_dir_all(&opt.output)?;
		}

		merger(&opt.input, &opt.output)?;

		opt.output
	};

	let elapsed = time.elapsed();
	println!("Finished merging resourcepacks in {:.3?}...", elapsed);
	println!("Output the result into '{}'", output.display());

	Ok(())
}

fn merger(input: &Path, output: &Path) -> Result<()> {
	let workspace = Workspace::from_path(input)?;
	let timeline = workspace.resolve();
	timeline.export_to(output)?;

	Ok(())
}

fn zip_dir(path: &Path, from: &Path) -> Result<()> {
	let file = File::create(path)?;
	let mut zipper = ZipWriter::new(file);
	zipper.create_from_directory(&from.to_path_buf())?; // Da fuck, zip-extensions!?
	Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Catalyst for merging resourcepack")]
pub struct Opt {
	/// Input directory
	#[structopt(parse(from_str))]
	input: PathBuf,

	/// Output directory
	#[structopt(parse(from_str))]
	output: PathBuf,

	/// Compress the output directory into zip file
	#[structopt(long, short)]
	zip: bool,
}
