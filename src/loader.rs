use std::{
    collections::HashMap,
    io::{BufReader, Seek},
    path::PathBuf,
};
use thiserror::Error;

use crate::{read, Binary, BinaryError, Format};

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("The specified binary path does not exist: {0}")]
    BinaryDoesNotExist(PathBuf),
    #[error("The binary file is missing or malformed: {0}")]
    FileNotFound(String),
    #[error("Failed to deserialize the binary format: {0}")]
    DeserializationFailed(#[from] BinaryError),
    #[error("Failed to read the binary file: {0}")]
    ReadError(#[from] std::io::Error),
}

pub struct Loader {
    binary: PathBuf,
    files: HashMap<String, (u64, u64)>,
    start: u64,
}

impl Loader {
    pub fn new(binary: impl Into<PathBuf>) -> Self {
        Loader {
            binary: binary.into(),
            files: HashMap::new(),
            start: 0,
        }
    }

    pub fn files(&self) -> Vec<String> {
        self.files.iter().map(|file| file.0.clone()).collect()
    }

    pub fn load(&mut self) -> Result<(), LoaderError> {
        if !self.binary.exists() {
            return Err(LoaderError::BinaryDoesNotExist(self.binary.clone()));
        }
        let file = std::fs::File::open(&self.binary)
            .map_err(|_| LoaderError::BinaryDoesNotExist(self.binary.clone()))?;
        let mut reader = BufReader::new(file);

        let format =
            Format::deserialize(&mut reader).map_err(LoaderError::DeserializationFailed)?;

        for file in format.files {
            println!("Found file: {:#?}", file);
            self.files
                .insert(file.path.clone(), (file.offset, file.size));
        }

        self.start = reader.stream_position().map_err(LoaderError::ReadError)?;

        Ok(())
    }

    pub fn read_raw(&mut self, path: &str) -> Result<Vec<u8>, LoaderError> {
        if let Some((offset, size)) = self.files.get(path) {
            let file = std::fs::File::open(&self.binary)
                .map_err(|_| LoaderError::BinaryDoesNotExist(self.binary.clone()))?;
            let mut reader = BufReader::new(file);

            read::skip(&mut reader, self.start + offset).map_err(LoaderError::ReadError)?;
            let mut buffer = vec![0; *size as usize];
            read::exact(&mut reader, &mut buffer).map_err(LoaderError::ReadError)?;
            Ok(buffer)
        } else {
            Err(LoaderError::FileNotFound(path.to_string()))
        }
    }

    pub fn read(&mut self, path: &str) -> Result<String, LoaderError> {
        let bytes = self.read_raw(path)?;
        String::from_utf8(bytes.into()).map_err(|e| {
            LoaderError::ReadError(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })
    }
}
