use std::io::{Read, Write};

use crate::{read, write, Binary, BinaryError};

const MAGIC: &str = "RASS";
const GITHUB: &str = "github.com/levilovie/rdss";

#[derive(Debug)]
pub struct Header {
    pub magic: String,
    pub github: String,
    pub version_major: u8,
    pub version_minor: u8,
    pub version_patch: u8,
}

impl Header {
    pub fn new() -> Self {
        let version = env!("CARGO_PKG_VERSION")
            .split('.')
            .map(|s| s.parse().unwrap_or(0))
            .collect::<Vec<u8>>();

        Header {
            magic: MAGIC.to_string(),
            github: GITHUB.to_string(),
            version_major: *version.get(0).unwrap_or(&0),
            version_minor: *version.get(1).unwrap_or(&0),
            version_patch: *version.get(2).unwrap_or(&0),
        }
    }

    pub fn check_version(&self) -> Result<(), BinaryError> {
        let version = env!("CARGO_PKG_VERSION")
            .split('.')
            .map(|s| s.parse().unwrap_or(0))
            .collect::<Vec<u8>>();
        let version_major = *version.get(0).unwrap_or(&0);
        let version_minor = *version.get(1).unwrap_or(&0);
        let version_patch = *version.get(2).unwrap_or(&0);
        if self.version_major != version_major
            || self.version_minor != version_minor
            || self.version_patch != version_patch
        {
            return Err(BinaryError::IncorrectVersion(
                format!("{}.{}.{}", version_major, version_minor, version_patch),
                format!(
                    "{}.{}.{}",
                    self.version_major, self.version_minor, self.version_patch
                ),
            ));
        }
        Ok(())
    }
}

impl Binary for Header {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), BinaryError> {
        write::string_raw(writer, &self.magic)?;
        write::u8(writer, 0)?;
        write::string_raw(writer, &self.github)?;
        write::u8(writer, self.version_major)?;
        write::u8(writer, self.version_minor)?;
        write::u8(writer, self.version_patch)?;
        Ok(())
    }

    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, BinaryError> {
        let magic = read::string_raw(reader, MAGIC.len() as u32)
            .map_err(|e| BinaryError::SyntaxError("Failed to read magic".into(), e.to_string()))?;
        let _ = read::u8(reader)?;
        let github = read::string_raw(reader, GITHUB.len() as u32).map_err(|e| {
            BinaryError::SyntaxError("Failed to read GitHub link".into(), e.to_string())
        })?;
        let version_major = read::u8(reader).map_err(|e| {
            BinaryError::SyntaxError("Failed to read major version".into(), e.to_string())
        })?;
        let version_minor = read::u8(reader).map_err(|e| {
            BinaryError::SyntaxError("Failed to read minor version".into(), e.to_string())
        })?;
        let version_patch = read::u8(reader).map_err(|e| {
            BinaryError::SyntaxError("Failed to read patch version".into(), e.to_string())
        })?;

        Ok(Header {
            magic,
            github,
            version_major,
            version_minor,
            version_patch,
        })
    }

    fn check(&self) -> Result<(), BinaryError> {
        if self.magic != MAGIC {
            return Err(BinaryError::SyntaxError(
                "Invalid magic number".into(),
                format!("Expected '{}', got '{}'", MAGIC, self.magic),
            ));
        }
        if self.github != GITHUB {
            return Err(BinaryError::SyntaxError(
                "Invalid GitHub link".into(),
                format!("Expected '{}', got '{}'", GITHUB, self.github),
            ));
        }
        self.check_version()
    }
}
