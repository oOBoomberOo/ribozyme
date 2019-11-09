mod model;
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

	/* let path = PathBuf::from("test/default.json");
	println!("exists?: {}", path.exists());
	let test = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("#1 Panicing..."));
	let data: model::BlockState = serde_json::from_str(&test).unwrap_or_else(|e| panic!("{}", e));
	println!("{:#?}", data); */
}