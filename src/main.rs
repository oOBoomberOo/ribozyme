#[macro_use]
extern crate clap;

use clap::App;
use std::path::PathBuf;

mod resourcepack;
mod resourcepack_meta;
use resourcepack::Resourcepack;
use resourcepack_meta::ResourcepackMeta;
use resourcepack::resources;

fn main() {
	let cli = load_yaml!("../resource/cli.yml");
	let app = App::from_yaml(cli);
	let matches = app.get_matches();

	if let Some(arg) = matches.value_of("directory") {
		let path = PathBuf::from(arg);

		if let Err(error) = run(path) {
			eprintln!("{}", error);
		}
	}
}

type ProgramResult<T> = Result<T, ProgramError>;

use console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Checkboxes, Input};
use std::error;
use std::io;
fn run(directory: PathBuf) -> ProgramResult<()> {
	if !directory.exists() {
		return Err(ProgramError::NotExists(directory));
	}

	if directory.is_file() {
		return Err(ProgramError::NotDirectory(directory));
	}

	let theme = ColorfulTheme::default();
	let terminal = Term::stdout();

	let resourcepacks = get_resourcepacks(&directory)?;

	terminal.write_line(
		&format!("Found {} resourcepacks.", resourcepacks.len())
	)?;

	let checked_resourcepacks: Vec<(&ResourcepackMeta, bool)> = resourcepacks.iter().map(|res| (res, true)).collect();

	let merge_list = Checkboxes::with_theme(&theme)
		.with_prompt("Select resourcepacks to merge")
		.paged(true)
		.items_checked(&checked_resourcepacks)
		.clear(true)
		.interact_on(&terminal)?;

	let mut passed_resourcepacks: Vec<ResourcepackMeta> = Vec::default();
	merge_list.iter()
		.for_each(|&index| {
			passed_resourcepacks.push(resourcepacks[index].clone());
		});
	
	let mut output_resourcepack: Resourcepack = Resourcepack::default();
	for resourcepack in passed_resourcepacks {
		let resourcepack = resourcepack.build()?;
		output_resourcepack.merge(resourcepack)?;
	}

	terminal.write_line("Successfully merged all resourcepacks.")?;

	let merged_name = Input::with_theme(&theme)
		.allow_empty(false)
		.show_default(true)
		.with_prompt("Merged Resourcepack name")
		.validate_with(PathValidator)
		.default(String::from("Ribozyme"))
		.interact_on(&terminal)?;

	let output_path = directory.join(
		format!("{}.zip", merged_name)
	);
	output_resourcepack.build(&output_path)?;

	terminal.write_line(
		&format!("Write merged resourcepacks to: {}", style(output_path.display()).cyan())
	)?;

	Ok(())
}

type Resourcepacks = Vec<ResourcepackMeta>;
fn get_resourcepacks(directory: &PathBuf) -> ProgramResult<Resourcepacks> {
	// let result: Result<Vec<_>, _> = directory.read_dir()?
	let result = directory.read_dir()?
		// .map(|entry| ResourcepackMeta::new(entry))
		.filter_map(|entry| ResourcepackMeta::new(entry).ok())
		.collect();
	Ok(result)
}

use zip::result::ZipError;
use std::path::StripPrefixError;
#[derive(Debug)]
pub enum ProgramError {
	NotDirectory(PathBuf),
	NotExists(PathBuf),
	NotResourcepack(PathBuf),
	NotValidNamespace(PathBuf),
	IoWithPath(PathBuf, io::Error),
	Io(io::Error),
	SerdeWithPath(PathBuf, serde_json::Error),
	Serde(serde_json::Error),
	Resource(resources::ResourceError),
	ZipWithPath(PathBuf, ZipError),
	Zip(ZipError),
	InvalidResourceFormat(PathBuf, resources::ResourceFormat),
	StripPrefix(StripPrefixError),
	Regex(regex::Error),
	Other(String)
}

use console::style;
use std::fmt;
impl fmt::Display for ProgramError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ProgramError::NotDirectory(path) => {
				write!(f, "'{}' is not a directory.", style(path.display()).cyan())
			}
			ProgramError::NotExists(path) => {
				write!(f, "'{}' does not exists.", style(path.display()).cyan())
			}
			ProgramError::NotResourcepack(path) => {
				write!(f, "'{}' is not a resourcepack.", style(path.display()).cyan())
			}
			ProgramError::NotValidNamespace(path) => {
				if path.is_file() {
					write!(f, "'{}' is not a valid file inside namespace.", style(path.display()).cyan())
				}
				else {
					write!(f, "'{}' is not a valid folder inside namespace.", style(path.display()).cyan())
				}
			}
			ProgramError::IoWithPath(path, error) => write!(f, "[{}] {}", style(path.display()).cyan(), error),
			ProgramError::SerdeWithPath(path, error) => write!(f, "[{}] {}", style(path.display()).cyan(), error),
			ProgramError::Serde(error) => write!(f, "{}", error),
			ProgramError::Resource(error) => write!(f, "{}", error),
			ProgramError::Io(error) => write!(f, "{}", style(error).red()),
			ProgramError::ZipWithPath(path, error) => write!(f, "[{}] {}.", style(path.display()).cyan(), error),
			ProgramError::Zip(error) => write!(f, "{}", error),
			ProgramError::InvalidResourceFormat(path, format) => write!(f, "'{}' is an invalid resource format: {:?}.", style(path.display()).cyan(), style(format).blue()),
			ProgramError::StripPrefix(error) => write!(f, "{}", error),
			ProgramError::Regex(error) => write!(f, "{}", error),
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

impl From<ZipError> for ProgramError {
	fn from(error: ZipError) -> ProgramError {
		ProgramError::Zip(error)
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

struct PathValidator;

use regex::Regex;
use dialoguer::Validator;
impl Validator for PathValidator {
	type Err = ProgramError;

	fn validate(&self, text: &str) -> Result<(), ProgramError> {
		let rex = Regex::new(r#"^[\w\-. ]+$"#)?;
		if rex.is_match(text) {
			Ok(())
		}
		else {
			Err(ProgramError::Other("Not a valid file name".to_owned()))
		}
	}
}