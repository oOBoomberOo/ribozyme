use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Incompatible file type for merging")]
	IncompatibleFile,
}
