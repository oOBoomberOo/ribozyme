use std::path::PathBuf;
use std::io::Result;
use colored::*;
use crate::gene::{Meta, Chromosome, ChromosomeKind};

pub fn merger(directory: &PathBuf) -> Result<()> {
	let output = directory.join(".out");
	let temp = directory.join(".temp");

	if output.exists() {
		fs::remove_dir_all(&output)?;
	}
	fs::create_dir_all(&output)?;

	if temp.exists() {
		fs::remove_dir_all(&temp)?;
	}
	fs::create_dir_all(&temp)?;

	let mut meta = Vec::default();
	for entry in fs::read_dir(&directory)? {
		let entry: fs::DirEntry = entry?;
		let chromosome = entry.path();
		let chromosome = Chromosome::new(&chromosome, &output);

		let chromosome_kind = chromosome.validate();
		if chromosome_kind == ChromosomeKind::Directory {
			meta.push(chromosome.handle()?);
		}
		else if chromosome_kind == ChromosomeKind::Compressed {
			let chromosome = extract(&chromosome.location, &temp)?;
			let chromosome = Chromosome::new(&chromosome, &output);
			meta.push(chromosome.handle()?);
		}
	}

	let total_meta: Meta = meta.into_iter().fold(Meta::default(), |sum, current| sum + current);

	println!("{} Successfully combined all resourcepacks ({} bytes)", "[✓]".green(), total_meta.bytes);
	println!("{} Total files: {}", "[✓]".green(), format!("{}", total_meta.nucleotides).green());
	println!("{} Total duplicates: {}", "[✓]".green(), format!("{}", total_meta.duplicates).green());
	println!("{} Total conflicts: {}", "[✓]".green(), format!("{}", total_meta.conflicts).green());

	let zip_output = PathBuf::from("ribozyme.zip");
	println!("{} Compressing file to {}", "[~]".yellow(), format!("'{}'", zip_output.display()).cyan());
	let total_bytes = compress(&output, &zip_output)?;
	println!("{} Finish compressing file to {} ({} bytes)", "[✓]".green(), format!("'{}'", zip_output.display()).cyan(), total_bytes);
	println!("{} P.S. total bytes is not accurate for some reason and I have no idea why but it's pretty close", "[?]".green());

	println!("{} Removing temp directory", "[~]".yellow());
	fs::remove_dir_all(&temp)?;
	fs::remove_dir_all(&output)?;
	println!("{} Finish removing temp directory", "[✓]".green());

	Ok(())
}

use std::fs;
use std::fs::File;
use zip::ZipArchive;
fn extract(path: &PathBuf, temp: &PathBuf) -> Result<PathBuf> {
	let file = File::open(&path)?;
	let mut zipper = ZipArchive::new(file)?;
	let chromosome = temp.join(&path);

	for i in 0..zipper.len() {
		let mut file = zipper.by_index(i)?;
		let outpath: PathBuf = chromosome.join(file.sanitized_name());

		if file.name().ends_with("/") {
			fs::create_dir_all(&outpath)?;
		}
		else {
			if let Some(parent) = outpath.parent() {
				if !parent.exists() {
					fs::create_dir_all(&parent)?;
				}
			}
			let mut outfile = File::create(&outpath)?;
			std::io::copy(&mut file, &mut outfile)?;
		}
	}

	Ok(chromosome)
}

use zip::{ZipWriter, CompressionMethod};
use zip::write::FileOptions;
use std::path::Path;
use std::io::{Write, Read};

use walkdir::{WalkDir, DirEntry};

fn compress(from: &PathBuf, to: &PathBuf) -> Result<usize> {
	let walker = WalkDir::new(&from);
	let writer = File::create(&to)?;
	let mut zipper = ZipWriter::new(writer);
	let options = FileOptions::default().compression_method(CompressionMethod::Bzip2).unix_permissions(0o755);
	let mut buffer = Vec::new();
	let mut total_bytes = 0;

	for entry in walker.into_iter() {
		let entry: DirEntry = entry?;
		let path: &Path = entry.path();
		let name = path.strip_prefix(Path::new(from)).unwrap();

		if path.is_file() {
			zipper.start_file_from_path(name, options)?;
			let mut file = File::open(&path)?;
			file.read_to_end(&mut buffer)?;
			total_bytes += zipper.write(&buffer)?;
			buffer.clear();
		}
		else if name.as_os_str().len() != 0 {
			zipper.add_directory_from_path(name, options)?;
		}
	}

	zipper.finish()?;
	Ok(total_bytes)
}