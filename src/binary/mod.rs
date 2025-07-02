pub mod read;
pub mod write;

use std::io::{Read, Write};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BinaryError {
    #[error("Failed to read/write binary stream: {0}")]
    IOFailed(std::io::Error),
    #[error("Invalid binary data encountered {0}: {1}")]
    SyntaxError(String, String),
    #[error("Binary data version mismatch: expected {0}, got {1}")]
    IncorrectVersion(String, String),
}

impl From<std::io::Error> for BinaryError {
    fn from(err: std::io::Error) -> Self {
        BinaryError::IOFailed(err)
    }
}

pub trait Binary {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), BinaryError>;
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, BinaryError>
    where
        Self: Sized;
    fn check(&self) -> Result<(), BinaryError> {
        Ok(())
    }
}
