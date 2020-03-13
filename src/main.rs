use clap::{crate_authors, crate_name, crate_version, App, Arg};
use std::path::PathBuf;

mod resourcepack;
mod resourcepack_meta;
mod utils;
mod error;

use error::{ProgramError, ProgramResult};
use resourcepack::resources;
use resourcepack::Resourcepack;
use resourcepack_meta::{MetaError, ResourcepackMeta};
use utils::ResourcePath;

fn main() {
	if let Err(error) = run() {
		eprintln!("{}", error);
	}
}

use dialoguer::theme::ColorfulTheme;
use dialoguer::{Checkboxes, Input};
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::io;
fn run() -> ProgramResult<()> {
	let app = init_cli_app();
	let matches = app.get_matches();
	let directory = get_directory(matches.value_of("directory"))?;
	let compression_level = get_compression_level(matches.value_of("compression_level"))?;

	if !directory.exists() {
		return Err(ProgramError::NotExists(directory));
	}
	if directory.is_file() {
		return Err(ProgramError::NotDirectory(directory));
	}

	let theme = ColorfulTheme::default();
	let resourcepacks = get_resourcepacks(&directory)?;

	println!("Found {} resourcepacks", resourcepacks.len());

	let selection_list = ask_resourcepack_selection(&resourcepacks, &theme)?;
	let resourcepack_list: Vec<ResourcepackMeta> = selection_list
		.iter()
		.map(|&index| resourcepacks[index].clone())
		.collect();

	let merged_name = ask_resourcepack_name(&theme)?;
	let output_path = PathBuf::from(format!("{}.tar.gz", merged_name));

	let mut output_resourcepack = Resourcepack::new(&output_path);

	let resourcepacks: Result<Vec<Resourcepack>, _> = resourcepack_list
		.into_par_iter()
		.map(|meta| meta.build())
		.collect();

	let resourcepacks = resourcepacks?;
	let merging_progress = create_merging_progress_bar(&resourcepacks);
	for resourcepack in resourcepacks {
		output_resourcepack.merge(resourcepack, &merging_progress)?;
	}
	merging_progress.finish();

	let compressing_progress = create_compressing_progress_bar(&output_resourcepack);
	output_resourcepack.build(&compressing_progress, compression_level)?;
	compressing_progress.finish();

	let meta = fs::metadata(&output_path)?;
	let size = meta.len();
	println!(
		"Merged resourcepacks into '{}' ({})",
		output_path.display(),
		HumanBytes(size)
	);

	Ok(())
}

fn create_merging_progress_bar(resourcepacks: &[Resourcepack]) -> ProgressBar {
	let size: u64 = resourcepacks.iter().map(Resourcepack::count).sum();
	ProgressBar::new(size).with_style(
		ProgressStyle::default_bar().template("{spinner:.cyan} Merging... {wide_bar:.cyan/white} {pos}/{len}"),
	)
}

fn create_compressing_progress_bar(resourcepack: &Resourcepack) -> ProgressBar {
	let size = resourcepack.count();
	ProgressBar::new(size).with_style(
		ProgressStyle::default_bar().template("{spinner:.cyan} Compressing... {wide_msg:.white} {percent}% {pos}/{len}"),
	)
}

fn ask_resourcepack_selection(resourcepacks: &[ResourcepackMeta], theme: &ColorfulTheme) -> Result<Vec<usize>, io::Error> {
	Checkboxes::with_theme(theme)
		.with_prompt("Select resourcepack to merge")
		.paged(true)
		.items_checked(&checked_resourcepacks(&resourcepacks))
		.interact()
}

fn ask_resourcepack_name(theme: &ColorfulTheme) -> Result<String, io::Error> {
	Input::<String>::with_theme(theme)
		.show_default(true)
		.allow_empty(false)
		.with_prompt("Merged file name")
		.default(String::from("ribozyme"))
		.validate_with(PathValidator)
		.interact()
}

fn init_cli_app<'a>() -> App<'a, 'a> {
	App::new(crate_name!())
		.version(crate_version!())
		.author(crate_authors!())
		.about("Catalyst for merging your resourcepacks")
		.arg(
			Arg::with_name("directory")
				.short("d")
				.long("path")
				.required(true)
				.index(1)
				.help("Directory to merge resourcepacks"),
		)
		.arg(
			Arg::with_name("compression_level")
				.short("l")
				.long("level")
				.takes_value(true)
				.help("Compression level of the outputed file (0-9)"),
		)
}

const DEFAULT_COMPRESSION_LEVEL: u32 = 9;
fn get_compression_level(input: Option<&str>) -> ProgramResult<u32> {
	let result = match input {
		None => Ok(0),
		Some(result) => result.parse::<u32>()
	};

	if let Ok(result) = result {
		if result > 9 {
			return Err(ProgramError::CompressionLevelTooLarge(result));
		}

		Ok(result)
	}
	else {
		Ok(DEFAULT_COMPRESSION_LEVEL)
	}
}

fn get_directory(input: Option<&str>) -> ProgramResult<PathBuf> {
	let directory = match input {
		Some(result) => result,
		None => unreachable!()
	};
	let result = PathBuf::from(directory);
	Ok(result)
}

fn checked_resourcepacks(resourcepacks: &[ResourcepackMeta]) -> Vec<(&ResourcepackMeta, bool)> {
	resourcepacks.iter().map(|x| (x, true)).collect()
}

use std::iter::Iterator;
fn get_resourcepacks(directory: &PathBuf) -> ProgramResult<Vec<ResourcepackMeta>> {
	let result = directory
		.read_dir()?
		.par_bridge()
		.filter_map(|entry| ResourcepackMeta::new(entry).ok())
		.collect();
	Ok(result)
}

struct PathValidator;

use dialoguer::Validator;
use regex::Regex;
impl Validator for PathValidator {
	type Err = ProgramError;

	fn validate(&self, text: &str) -> Result<(), ProgramError> {
		let rex = Regex::new(r#"^[\w\-. ]+$"#)?;
		if rex.is_match(text) {
			Ok(())
		} else {
			Err(ProgramError::Other("Not a valid file name".to_owned()))
		}
	}
}
