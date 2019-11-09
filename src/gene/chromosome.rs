use std::path::PathBuf;
use std::io::Result;
use std::fs;
use colored::*;
use super::{Nucleotide, Meta, Format, FormatKind};

/**
 * Chromosome contain many nucleotides in a single struct
*/
pub struct Chromosome {
	pub location: PathBuf,
	pub output: PathBuf,
}

impl Chromosome {
	/**
	 * Create new chromosome
	 *  
	 * location: `&PathBuf` of where the chromosome is  
	 * output: `&PathBuf` of where the output is  
	*/
	pub fn new(location: &PathBuf, output: &PathBuf) -> Chromosome {
		let location = location.to_owned();
		let output = output.to_owned();
		Chromosome { location, output }
	}

	/**
	 * Check if the chromosome is valid (contains `/assets/` and `/pack.mcmeta` or chromosome is a zip file)
	*/
	pub fn validate(&self) -> ChromosomeKind {
		let mut result = ChromosomeKind::Invalid;
		if self.location.is_dir() && self.location != self.output {
			let pack = self.location.join("pack.mcmeta");
			let assets = self.location.join("assets");
			let parent = self.parent();
			if pack.exists() && assets.exists() && parent != ".temp" {
				result = ChromosomeKind::Directory;
			}
		}
		else if let Some(extension) = self.location.extension() {
			if extension == "zip" {
				result = ChromosomeKind::Compressed;
			}
		}

		result
	}

	/**
	 * Handle merging of chromosome
	*/
	pub fn handle(&self) -> Result<Meta> {
		println!("{} Start working on {}", "[~]".yellow(), format!("'{}'", self.location.display()).cyan());
		let nucleotides = Chromosome::stalker(&self.location, &self)?;
		let mut result: Meta = nucleotides.iter().fold(Meta::default(), |sum, nucleotide| sum + nucleotide.handle());
		result.nucleotides = nucleotides.len();
		println!("{} Finish working on {}", "[~]".yellow(), format!("'{}'", self.location.display()).cyan());
		Ok(result)
	}

	/**
	 * Get chromosome's parent name
	*/
	fn parent(&self) -> &str {
		let parent = match self.location.parent() {
			Some(x) => match x.file_name() {
				Some(y) => match y.to_str() {
					Some(z) => z,
					None => panic!("Unable to convert parent's name to &str"),
				},
				None => panic!("Unable to get parent's name"),
			},
			None => panic!("Unable to get parent location"),
		};
		parent
	}

	/**
	 * Recursively walk through chromosome's directory and return EVERY files inside except file with format `FormatKind::Skip`
	*/
	fn stalker(path: &PathBuf, chromosome: &Chromosome) -> Result<Vec<Nucleotide>> {
		let mut nucleotides = Vec::default();
		for entry in fs::read_dir(&path)? {
			let entry: fs::DirEntry = entry?;
			let child = entry.path();
			if child.is_file() {
				let format = Format::identify_format(&child);
				if format.kind != FormatKind::Skip {
					let nucleotide = Nucleotide::new(&child, &chromosome);
					nucleotides.push(nucleotide);
				}
			}
			else if child.is_dir() {
				let mut more_nucleotides = Chromosome::stalker(&child, &chromosome)?;
				nucleotides.append(&mut more_nucleotides);
			}
		}
		Ok(nucleotides)
	}
}

#[derive(PartialEq, Eq)]
pub enum ChromosomeKind {
	Directory,
	Compressed,
	Invalid,
}