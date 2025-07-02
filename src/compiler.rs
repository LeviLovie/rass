use std::path::PathBuf;
use thiserror::Error;

use crate::{write, Binary, File, Format};

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
    #[error("The specified sources path does not exist")]
    SourcesDoNotExist,
    #[error("Failed to read source {1}: {0}")]
    FailedReadSource(std::io::Error, PathBuf),
    #[error("Failed to get binary parent")]
    FailedGetBinaryParent,
    #[error("Failed to create binary: {0}")]
    FailedCreateBinary(std::io::Error),
    #[error("Failed to write {1}: {0}")]
    FailedWrite(PathBuf, String),
    #[error("Failed to open binary {1}: {0}")]
    FailedOpenBinary(std::io::Error, PathBuf),
    #[error("Failed to write contents: {0}")]
    FailedWriteContents(std::io::Error),
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
            .map_err(|e| CompilerError::FailedOpenBinary(e, self.binary.clone()))?;
        let mut writer = std::io::BufWriter::new(&mut file);

        let mut format = Format::new();
        let sources_raw = self.list_sources()?;
        let mut contents = Vec::new();

        let mut index: u64 = 0;
        for source in &sources_raw {
            let content = std::fs::read_to_string(&source)
                .map_err(|e| CompilerError::FailedReadSource(e, source.clone()))?;
            let size: u64 = content.len() as u64;
            let path = Self::relative_path(source, &self.sources).ok_or_else(|| {
                CompilerError::FailedReadSource(
                    std::io::Error::new(std::io::ErrorKind::NotFound, "Path not found"),
                    source.clone(),
                )
            })?;
            format.add_file(File::new(path, index, size));
            contents.push(content);
            index += size;
        }

        format
            .serialize(&mut writer)
            .map_err(|e| CompilerError::FailedWrite(self.binary.clone(), e.to_string()))?;

        for source in sources_raw {
            let content = std::fs::read_to_string(&source)
                .map_err(|e| CompilerError::FailedReadSource(e, source.clone()))?;
            write::string_raw(&mut writer, &content)
                .map_err(|e| CompilerError::FailedWriteContents(e))?;
        }

        Ok(())
    }

    pub fn check_files_exist(&self) -> Result<(), CompilerError> {
        self.sources
            .exists()
            .then_some(())
            .ok_or_else(|| CompilerError::SourcesDoNotExist)?;

        let binary_parent = self
            .binary
            .parent()
            .ok_or_else(|| CompilerError::FailedGetBinaryParent)?;
        if !binary_parent.exists() {
            std::fs::create_dir_all(binary_parent)
                .map_err(|e| CompilerError::FailedCreateBinary(e))?;
        }
        if self.binary.exists() {
            std::fs::remove_file(&self.binary)
                .map_err(|e| CompilerError::FailedOpenBinary(e, self.binary.clone()))?;
        }
        std::fs::File::create(&self.binary)
            .map_err(|e| CompilerError::FailedOpenBinary(e, self.binary.clone()))?;

        Ok(())
    }

    pub fn list_sources(&self) -> Result<Vec<PathBuf>, CompilerError> {
        if !self.sources.exists() {
            return Err(CompilerError::SourcesDoNotExist);
        }

        Ok(Self::list_files(&self.sources)?)
    }

    fn relative_path(source: &PathBuf, base: &PathBuf) -> Option<String> {
        source
            .strip_prefix(base)
            .ok()
            .map(|p| p.to_string_lossy().into_owned())
    }

    fn list_files(dir: &PathBuf) -> Result<Vec<PathBuf>, CompilerError> {
        let mut sources = Vec::new();
        for entry in
            std::fs::read_dir(dir).map_err(|e| CompilerError::FailedReadSource(e, dir.clone()))?
        {
            let entry = entry.map_err(|_| CompilerError::SourcesDoNotExist)?;
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
