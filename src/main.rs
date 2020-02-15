#[macro_use]
extern crate clap;

use clap::App;
use std::path::PathBuf;

fn main() {
	let cli = load_yaml!("../resource/cli.yml");
	let app = App::from_yaml(cli);
	let matches = app.get_matches();

	if let Some(arg) = matches.value_of("directory") {
		let path = PathBuf::from(arg);

		if let Err(error) = run(&path) {
			eprintln!("{}", error);
		}
	}
}

use std::error;
fn run(directory: &PathBuf) -> Result<(), Box<dyn error::Error>> {
	todo!()
}