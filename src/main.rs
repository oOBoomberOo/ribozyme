mod model;
mod rna;
mod gene;

use clap::{Arg, App};
use std::path::PathBuf;
use std::fs::canonicalize;

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
	if let Some(directory) = matches.value_of("directory") {
		let directory = PathBuf::from(directory);
		if let Ok(directory) = canonicalize(directory) {
			if directory.is_dir() {
				if let Err(_) = rna::merger(&directory) {
					println!("Unexpected error");
				}
			}
			else {
				println!("Given path is not directory");
			}
		}
		else {
			println!("Given path does not exists");
		}
	}
	else {
		println!("Invalid directory path");
	}
}