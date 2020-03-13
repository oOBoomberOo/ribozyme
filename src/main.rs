use clap::{crate_authors, crate_name, crate_version, App, Arg};
use std::path::PathBuf;

mod resourcepack;
mod resourcepack_meta;
mod utils;

use resourcepack::resources;
use resourcepack::Resourcepack;
use resourcepack_meta::{MetaError, ResourcepackMeta};
use utils::ResourcePath;

fn main() {
	if let Err(error) = run() {
		eprintln!("{}", error);
	}
}

const COMPRESSION_LEVEL: u32 = 9;

type ProgramResult<T> = Result<T, ProgramError>;

use colored::*;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Checkboxes, Input};
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::error;
use std::fs;
use std::io;
fn run() -> ProgramResult<()> {
	let app = App::new(crate_name!())
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
		);
	let matches = app.get_matches();
	let directory = match matches.value_of("directory") {
		Some(directory) => directory,
		None => unreachable!(),
	};
	let compression_level = match matches.value_of("compression_level") {
		Some(level) => {
			if let Ok(result) = level.parse::<u32>() {
				result
			} else {
				COMPRESSION_LEVEL
			}
		}
		None => COMPRESSION_LEVEL,
	};

	if compression_level > 9 {
		return Err(ProgramError::CompressionLevelTooLarge(compression_level));
	}

	let directory = PathBuf::from(directory);

	if !directory.exists() {
		return Err(ProgramError::NotExists(directory));
	}

	if directory.is_file() {
		return Err(ProgramError::NotDirectory(directory));
	}

	let theme = ColorfulTheme::default();
	let resourcepacks = get_resourcepacks(&directory)?;

	println!("Found {} resourcepacks", resourcepacks.len());

	let resourcepacks_checked: Vec<(_, _)> = resourcepacks
		.iter()
		.map(|resourcepack| (resourcepack, true))
		.collect();
	let selection_list = Checkboxes::with_theme(&theme)
		.with_prompt("Select resourcepack to merge")
		.paged(true)
		.items_checked(&resourcepacks_checked)
		.interact()?;

	let resourcepack_list: Vec<ResourcepackMeta> = selection_list
		.iter()
		.map(|&index| resourcepacks[index].clone())
		.collect();

	let merged_name = Input::<String>::with_theme(&theme)
		.show_default(true)
		.allow_empty(false)
		.with_prompt("Merged file name")
		.default(String::from("ribozyme"))
		.validate_with(PathValidator)
		.interact()?;
	let output_path = PathBuf::from(format!("{}.tar.gz", merged_name));

	let mut output_resourcepack = Resourcepack::new(&output_path);

	let resourcepacks: Result<Vec<Resourcepack>, _> = resourcepack_list
		.into_par_iter()
		.map(|meta| meta.build())
		.collect();

	let resourcepacks = resourcepacks?;

	let size: u64 = resourcepacks.iter().map(Resourcepack::count).sum();
	let merging_progress = ProgressBar::new(size).with_style(
		ProgressStyle::default_bar().template("Merging...     {wide_bar:.cyan/white} {pos}/{len}"),
	);

	for resourcepack in resourcepacks {
		output_resourcepack.merge(resourcepack, &merging_progress)?;
	}

	merging_progress.finish();

	let size = output_resourcepack.count();
	let compressing_progress = ProgressBar::new(size).with_style(
		ProgressStyle::default_bar().template("Compressing... {wide_bar:.cyan/white} {pos}/{len}"),
	);

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

use std::iter::Iterator;
fn get_resourcepacks(directory: &PathBuf) -> ProgramResult<Vec<ResourcepackMeta>> {
	let result = directory
		.read_dir()?
		.par_bridge()
		.filter_map(|entry| ResourcepackMeta::new(entry).ok())
		.collect();
	Ok(result)
}

use std::path::StripPrefixError;
#[derive(Debug)]
pub enum ProgramError {
	CompressionLevelTooLarge(u32),

	NotDirectory(PathBuf),
	NotExists(PathBuf),
	NotResourcepack(PathBuf),
	NotValidNamespace(PathBuf),

	IoWithPath(PathBuf, io::Error),
	Io(io::Error),

	SerdeWithPath(PathBuf, serde_json::Error),
	Serde(serde_json::Error),

	Resource(resources::ResourceError),

	InvalidResourceFormat(PathBuf, resources::ResourceFormat),

	StripPrefix(StripPrefixError),

	Regex(regex::Error),

	Meta(MetaError),

	Other(String),
}

use std::fmt;
impl fmt::Display for ProgramError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ProgramError::CompressionLevelTooLarge(value) => {
				write!(f, "Compression level {} is too large", value)
			}
			ProgramError::NotDirectory(path) => write!(
				f,
				"'{}' is not a directory.",
				path.display().to_string().cyan()
			),
			ProgramError::NotExists(path) => write!(
				f,
				"'{}' does not exists.",
				path.display().to_string().cyan()
			),
			ProgramError::NotResourcepack(path) => write!(
				f,
				"'{}' is not a resourcepack.",
				path.display().to_string().cyan()
			),
			ProgramError::NotValidNamespace(path) => {
				if path.is_file() {
					write!(
						f,
						"'{}' is not a valid file inside namespace.",
						path.display().to_string().cyan()
					)
				} else {
					write!(
						f,
						"'{}' is not a valid folder inside namespace.",
						path.display().to_string().cyan()
					)
				}
			}
			ProgramError::Io(error) => write!(f, "{}", error.to_string().red()),
			ProgramError::IoWithPath(path, error) => {
				write!(f, "[{}] {}", path.display().to_string().cyan(), error)
			}
			ProgramError::SerdeWithPath(path, error) => {
				write!(f, "[{}] {}", path.display().to_string().cyan(), error)
			}
			ProgramError::Serde(error) => write!(f, "{}", error),
			ProgramError::Resource(error) => write!(f, "{}", error),
			ProgramError::InvalidResourceFormat(path, format) => write!(
				f,
				"'{}' is an invalid resource format: {}.",
				path.display().to_string().cyan(),
				format!("{}", format).blue()
			),
			ProgramError::StripPrefix(error) => write!(f, "{}", error),
			ProgramError::Regex(error) => write!(f, "{}", error),
			ProgramError::Meta(error) => write!(f, "{}", error),
			ProgramError::Other(error) => write!(f, "{}", error),
		}
	}
}

impl error::Error for ProgramError {}

impl From<resources::ResourceError> for ProgramError {
	fn from(error: resources::ResourceError) -> ProgramError {
		ProgramError::Resource(error)
	}
}

impl From<io::Error> for ProgramError {
	fn from(error: io::Error) -> ProgramError {
		ProgramError::Io(error)
	}
}

impl From<serde_json::Error> for ProgramError {
	fn from(error: serde_json::Error) -> ProgramError {
		ProgramError::Serde(error)
	}
}

impl From<StripPrefixError> for ProgramError {
	fn from(error: StripPrefixError) -> ProgramError {
		ProgramError::StripPrefix(error)
	}
}

impl From<regex::Error> for ProgramError {
	fn from(error: regex::Error) -> ProgramError {
		ProgramError::Regex(error)
	}
}

impl From<MetaError> for ProgramError {
	fn from(error: MetaError) -> ProgramError {
		ProgramError::Meta(error)
	}
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
