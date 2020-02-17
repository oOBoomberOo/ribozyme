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
use dialoguer::Checkboxes;
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
	
	let mut output_resourcepack = Resourcepack::default();
	for resourcepack in passed_resourcepacks {
		let resourcepack = resourcepack.build()?;
		output_resourcepack.merge(resourcepack)?;
	}

	println!("{:#?}", output_resourcepack);

	Ok(())
}

type Resourcepacks = Vec<ResourcepackMeta>;
fn get_resourcepacks(directory: &PathBuf) -> ProgramResult<Resourcepacks> {
	let result = directory.read_dir()?
		.filter_map(|entry| ResourcepackMeta::new(entry).ok())
		.collect();
	Ok(result)
}

#[derive(Debug)]
pub enum ProgramError {
	NotDirectory(PathBuf),
	NotExists(PathBuf),
	NotResourcepack(PathBuf),
	NotValidNamespace(PathBuf),
	IoWithPath(PathBuf, io::Error),
	Io(io::Error),
	Serde(PathBuf, serde_json::Error),
	Resource(resources::ResourceError)
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
			ProgramError::IoWithPath(path, error) => write!(f, "[{}] {}.", style(path.display()).cyan(), error),
			ProgramError::Serde(path, error) => write!(f, "[{}] {}.", style(path.display()).cyan(), error),
			ProgramError::Resource(error) => write!(f, "{}.", error),
			ProgramError::Io(error) => write!(f, "{}.", error),
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