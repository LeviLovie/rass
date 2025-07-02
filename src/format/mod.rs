mod file;
mod header;

pub use file::File;
pub use header::Header;

use std::io::{Read, Write};

use crate::{read, write, Binary, BinaryError};

#[derive(Debug)]
pub struct Format {
    pub header: Header,
    pub files: Vec<File>,
}

impl Format {
    pub fn new() -> Self {
        Self {
            header: Header::new(),
            files: Vec::new(),
        }
    }

    pub fn add_file(&mut self, file: File) {
        self.files.push(file);
    }

    pub fn add_files(&mut self, files: Vec<File>) {
        self.files.extend(files);
    }
}

impl Binary for Format {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), BinaryError> {
        self.header.serialize(writer)?;

        write::u32(writer, self.files.len() as u32)?;
        for file in &self.files {
            file.serialize(writer)?;
        }
        Ok(())
    }

    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, BinaryError> {
        let header = Header::deserialize(reader)?;

        let mut files = Vec::new();
        let file_count = read::u32(reader)? as usize;
        for _ in 0..file_count {
            files.push(File::deserialize(reader)?);
        }
        Ok(Format { header, files })
    }

    fn check(&self) -> Result<(), BinaryError> {
        self.header.check()?;
        Ok(())
    }
}
