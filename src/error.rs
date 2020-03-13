use super::{MetaError, resources};
use std::path::{PathBuf, StripPrefixError};
use std::io;
use std::error;
use colored::*;

pub type ProgramResult<T> = Result<T, ProgramError>;

#[derive(Debug)]
pub enum ProgramError {
	CompressionLevelTooLarge(u32),

	NotDirectory(PathBuf),
	NotExists(PathBuf),

	IoWithPath(PathBuf, io::Error),
	Io(io::Error),

	SerdeWithPath(PathBuf, serde_json::Error),
	Serde(serde_json::Error),

	Resource(resources::ResourceError),

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
			ProgramError::Io(error) => write!(f, "{}", error.to_string().red()),
			ProgramError::IoWithPath(path, error) => {
				write!(f, "[{}] {}", path.display().to_string().cyan(), error)
			}
			ProgramError::SerdeWithPath(path, error) => {
				write!(f, "[{}] {}", path.display().to_string().cyan(), error)
			}
			ProgramError::Serde(error) => write!(f, "{}", error),
			ProgramError::Resource(error) => write!(f, "{}", error),
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

#[macro_export]
macro_rules! catch_io_error {
	($x:expr, $p:expr) => {
		match $x {
			Ok(result) => result,
			Err(error) => return Err(ProgramError::IoWithPath($p, error))
		}
	};
	($x:expr) => {
		match $x {
			Ok(result) => result,
			Err(error) => return Err(ProgramError::Io(error))
		}
	};
}