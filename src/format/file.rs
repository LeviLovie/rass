use std::io::{Read, Write};

use crate::{read, write, Binary, BinaryError};

#[derive(Debug)]
pub struct File {
    pub path: String,
    pub offset: u64,
    pub size: u64,
}

impl File {
    pub fn new(path: String, offset: u64, size: u64) -> Self {
        Self { path, offset, size }
    }
}

impl Binary for File {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), BinaryError> {
        write::u64(writer, self.offset)?;
        write::u64(writer, self.size)?;
        write::string(writer, &self.path)?;
        Ok(())
    }

    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, BinaryError> {
        let offset = read::u64(reader)?;
        let size = read::u64(reader)?;
        let path = read::string(reader)?;
        Ok(File { path, offset, size })
    }
}
