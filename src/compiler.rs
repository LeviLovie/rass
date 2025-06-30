use base64::prelude::*;
use std::{io::prelude::*, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilerBuilderError {
    #[error("Invalid path provided for sources")]
    NoSourcesPath,
    #[error("Invalid path provided for binary output")]
    NoBinaryPath,
}

pub struct CompilerBuilder {
    sources: Option<PathBuf>,
    binary: Option<PathBuf>,
}

impl Default for CompilerBuilder {
    fn default() -> Self {
        CompilerBuilder {
            sources: None,
            binary: None,
        }
    }
}

impl CompilerBuilder {
    pub fn from_sources(mut self, sources: impl Into<PathBuf>) -> Self {
        self.sources = Some(sources.into());
        self
    }

    pub fn save_to(mut self, binary: impl Into<PathBuf>) -> Self {
        self.binary = Some(binary.into());
        self
    }

    pub fn build(self) -> Result<Compiler, CompilerBuilderError> {
        let sources = self.sources.ok_or(CompilerBuilderError::NoSourcesPath)?;
        let binary = self.binary.ok_or(CompilerBuilderError::NoBinaryPath)?;

        Ok(Compiler { sources, binary })
    }
}

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("The specified sources directory does not exist: {0}")]
    SourcesDirDoesNotExist(PathBuf),
    #[error("The specified binary path does not exist")]
    BinaryFailedToCreate,
    #[error("Failed to write to the binary file at {0}: {1}")]
    FailedToWrite(PathBuf, String),
    #[error("Failed to open the binary file at {0}: {1}")]
    FailedToOpenBinary(PathBuf, String),
    #[error("Failed to read the sources directory: {0}")]
    SourcesEntryDoesNotExist(PathBuf),
    #[error("Failed to read the source file at {0}: {1}")]
    FailedToReadSource(PathBuf, String),
}

pub struct Compiler {
    sources: PathBuf,
    binary: PathBuf,
}

impl Compiler {
    pub fn builder() -> CompilerBuilder {
        CompilerBuilder::default()
    }

    pub fn compile(&self) -> Result<(), CompilerError> {
        self.check_files_exist()?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(self.binary.clone())
            .map_err(|e| CompilerError::FailedToOpenBinary(self.binary.clone(), e.to_string()))?;

        let mut index: u32 = 0;
        self.write_bin(&mut file, &mut index, "RASS v")?;
        self.write_bin(&mut file, &mut index, env!("CARGO_PKG_VERSION"))?;
        self.write_bin(&mut file, &mut index, " (github.com/LeviLovie/rass)\n")?;

        let sources = self.list_sources()?;
        let mut contents: Vec<String> = Vec::new();
        let mut sources_vec: Vec<(String, u32, u32)> = Vec::new();
        let mut sources_index: u32 = 0;
        for source in sources {
            let content =
                BASE64_STANDARD.encode(std::fs::read_to_string(&source).map_err(|e| {
                    CompilerError::FailedToReadSource(source.clone(), e.to_string())
                })?);
            contents.push(content.clone());
            sources_vec.push((
                BASE64_STANDARD.encode(source.display().to_string()),
                sources_index,
                content.len() as u32,
            ));
            sources_index += content.len() as u32;
        }

        for source in &sources_vec {
            self.write_bin(
                &mut file,
                &mut index,
                &format!("<{}:{}:{}>", source.0, source.1, source.2),
            )?;
        }
        self.write_bin(&mut file, &mut index, "&")?;

        for content in &contents {
            self.write_bin(&mut file, &mut index, content)?;
        }

        Ok(())
    }

    pub fn write_bin(
        &self,
        file: &mut std::fs::File,
        index: &mut u32,
        content: &str,
    ) -> Result<(), CompilerError> {
        *index += content.len() as u32;
        file.write_all(content.as_bytes())
            .map_err(|e| CompilerError::FailedToWrite(self.binary.clone(), e.to_string()))?;
        Ok(())
    }

    pub fn check_files_exist(&self) -> Result<(), CompilerError> {
        self.sources
            .exists()
            .then_some(())
            .ok_or_else(|| CompilerError::SourcesDirDoesNotExist(self.sources.clone()))?;

        let binary_parent = self
            .binary
            .parent()
            .ok_or_else(|| CompilerError::BinaryFailedToCreate)?;
        if !binary_parent.exists() {
            std::fs::create_dir_all(binary_parent)
                .map_err(|_| CompilerError::BinaryFailedToCreate)?;
        }
        if self.binary.exists() {
            std::fs::remove_file(&self.binary).map_err(|e| {
                CompilerError::FailedToOpenBinary(self.binary.clone(), e.to_string())
            })?;
        }
        std::fs::File::create(&self.binary)
            .map_err(|e| CompilerError::FailedToOpenBinary(self.binary.clone(), e.to_string()))?;

        Ok(())
    }

    pub fn list_sources(&self) -> Result<Vec<PathBuf>, CompilerError> {
        if !self.sources.exists() {
            return Err(CompilerError::SourcesDirDoesNotExist(self.sources.clone()));
        }

        Ok(Self::list_files(&self.sources)?)
    }

    fn list_files(dir: &PathBuf) -> Result<Vec<PathBuf>, CompilerError> {
        let mut sources = Vec::new();
        for entry in std::fs::read_dir(dir)
            .map_err(|_| CompilerError::SourcesEntryDoesNotExist(dir.clone()))?
        {
            let entry = entry.map_err(|_| CompilerError::SourcesEntryDoesNotExist(dir.clone()))?;
            let path = entry.path();
            if path.is_dir() {
                sources.extend(Self::list_files(&path)?);
            } else {
                sources.push(path);
            }
        }
        Ok(sources)
    }
}
