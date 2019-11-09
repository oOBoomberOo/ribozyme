mod rna;
mod gene;

use clap::{Arg, App};
use std::path::PathBuf;

fn main() {
	let matches = App::new("Ribozyme")
		.version("0.1.0")
		.author("Boomber")
		.about("Catalyst for resourcepacks merging")
		.arg(Arg::with_name("directory")
			.short("d")
			.help("Merge resourcepacks within this directory")
			.required(true)
			.index(1))
		.get_matches();
	let directory = matches.value_of("directory").unwrap_or_else(|| panic!("Invalid directory!"));
	let directory = PathBuf::from(directory);
	rna::merger(&directory).unwrap();
}