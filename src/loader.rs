use base64::prelude::*;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::PathBuf,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("The specified binary path does not exist: {0}")]
    BinaryDoesNotExist(PathBuf),
    #[error("Failed to read the binary file: {0}: {1}")]
    SyntaxError(String, String),
    #[error("The binary file is missing or malformed: {0}")]
    FileNotFound(String),
}

pub struct Loader {
    binary: PathBuf,
    files: HashMap<String, (u32, u32)>,
}

impl Loader {
    pub fn new(binary: impl Into<PathBuf>) -> Self {
        Loader {
            binary: binary.into(),
            files: HashMap::new(),
        }
    }

    pub fn files(&self) -> &HashMap<String, (u32, u32)> {
        &self.files
    }

    pub fn load(&mut self) -> Result<(), LoaderError> {
        if !self.binary.exists() {
            return Err(LoaderError::BinaryDoesNotExist(self.binary.clone()));
        }
        let file = std::fs::File::open(&self.binary)
            .map_err(|_| LoaderError::BinaryDoesNotExist(self.binary.clone()))?;
        let mut reader = BufReader::new(file);

        if let Err(e) = Self::expect_next_chars(&mut reader, "RASS v") {
            return Err(LoaderError::SyntaxError(
                "Failed to read magic number".to_string(),
                e.to_string(),
            ));
        }

        match Self::read_next_chars(&mut reader, 5) {
            Ok(version) => {
                if version != env!("CARGO_PKG_VERSION") {
                    return Err(LoaderError::SyntaxError(
                        format!(
                            "Unsupported version: {}, expected: {}",
                            version,
                            env!("CARGO_PKG_VERSION")
                        ),
                        version,
                    ));
                }
            }
            Err(e) => {
                return Err(LoaderError::SyntaxError(
                    "Failed to read version".to_string(),
                    e.to_string(),
                ));
            }
        }

        if let Err(e) = Self::expect_next_chars(&mut reader, " (github.com/LeviLovie/rass)\n") {
            return Err(LoaderError::SyntaxError(
                "Failed to read link".to_string(),
                e.to_string(),
            ));
        }

        loop {
            if let Ok(next) = Self::peek_next_chars(&mut reader, 1) {
                if next == "&" {
                    break;
                }
            } else {
                return Err(LoaderError::SyntaxError(
                    "Failed to read end of header".to_string(),
                    "Unexpected end of reader".to_string(),
                ));
            }

            match Self::read_until(&mut reader, '>') {
                Err(e) => {
                    return Err(LoaderError::SyntaxError(
                        "Failed to read source header".to_string(),
                        e.to_string(),
                    ));
                }
                Ok(header) => {
                    let header = header.trim_start_matches("<").trim_end_matches(">");
                    let parts: Vec<&str> = header.split(':').collect();
                    if parts.len() != 3 {
                        return Err(LoaderError::SyntaxError(
                            "Invalid source header format".to_string(),
                            header.to_string(),
                        ));
                    }
                    let name = String::from_utf8(
                        BASE64_STANDARD.decode(parts[0].to_string()).map_err(|e| {
                            LoaderError::SyntaxError(
                                "Failed to decode source name".to_string(),
                                e.to_string(),
                            )
                        })?,
                    )
                    .map_err(|e| {
                        LoaderError::SyntaxError(
                            "Failed to convert source name to UTF-8".to_string(),
                            e.to_string(),
                        )
                    })?;
                    if self.files.contains_key(&name) {
                        return Err(LoaderError::SyntaxError(
                            "Duplicate source file found".to_string(),
                            name.to_string(),
                        ));
                    }
                    let offset = parts[1].parse::<u32>().map_err(|e| {
                        LoaderError::SyntaxError(
                            "Failed to parse source offset".to_string(),
                            e.to_string(),
                        )
                    })?;
                    let length = parts[2].parse::<u32>().map_err(|e| {
                        LoaderError::SyntaxError(
                            "Failed to parse source length".to_string(),
                            e.to_string(),
                        )
                    })?;
                    self.files.insert(name, (offset, length));
                }
            }
        }

        Ok(())
    }

    pub fn read(&self, file_name: &str) -> Result<String, LoaderError> {
        let (start, len) = self
            .files
            .get(file_name)
            .ok_or_else(|| LoaderError::FileNotFound(file_name.to_string()))?;
        let file = std::fs::File::open(&self.binary)
            .map_err(|_| LoaderError::BinaryDoesNotExist(self.binary.clone()))?;
        let mut reader = BufReader::new(file);

        if let Err(e) = Self::skip_after(&mut reader, '&') {
            return Err(LoaderError::SyntaxError(
                "Failed to read until end of header".to_string(),
                e.to_string(),
            ));
        }

        match Self::read_file(&mut reader, *start, *len) {
            Ok(data) => {
                let data = String::from_utf8(data).map_err(|e| {
                    LoaderError::SyntaxError("Invalid UTF-8 data".to_string(), e.to_string())
                })?;
                println!(
                    "Read {} bytes from file '{}': {:?}",
                    data.len(),
                    file_name,
                    data
                );
                let decoded = BASE64_STANDARD.decode(data).map_err(|e| {
                    LoaderError::SyntaxError("Failed to decode base64".to_string(), e.to_string())
                })?;
                let decoded_str = String::from_utf8(decoded).map_err(|e| {
                    LoaderError::SyntaxError(
                        "Failed to convert to UTF-8".to_string(),
                        e.to_string(),
                    )
                })?;
                Ok(decoded_str)
            }
            Err(e) => Err(LoaderError::SyntaxError(
                "Failed to read binary data".to_string(),
                e.to_string(),
            )),
        }
    }

    /// Reads the next `count` UTF-8 characters from a buffered reader.
    fn read_next_chars<R: BufRead>(reader: &mut R, count: usize) -> std::io::Result<String> {
        let mut result = String::new();
        let mut total_bytes = 0;

        loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break; // EOF
            }

            let text = std::str::from_utf8(buf).unwrap_or_default();
            let mut chars = text.chars();

            for c in &mut chars {
                if result.chars().count() >= count {
                    break;
                }
                result.push(c);
                total_bytes += c.len_utf8();
            }

            reader.consume(total_bytes);
            if result.chars().count() >= count {
                break;
            }
        }

        Ok(result)
    }

    /// Peeks the next `count` UTF-8 characters from a buffered reader without consuming them.
    fn peek_next_chars<R: BufRead>(reader: &mut R, count: usize) -> std::io::Result<String> {
        let mut peeked = String::new();

        let buf = reader.fill_buf()?;
        if buf.is_empty() {
            return Ok(peeked); // EOF
        }

        let text = std::str::from_utf8(buf).unwrap_or_default();
        let mut chars = text.chars();

        for _ in 0..count {
            if let Some(c) = chars.next() {
                peeked.push(c);
            } else {
                break;
            }
        }

        Ok(peeked)
    }

    /// Reads and verifies that the next characters in the stream match `expected`, else returns an error.
    fn expect_next_chars<R: BufRead>(reader: &mut R, expected: &str) -> Result<(), String> {
        let actual =
            Self::read_next_chars(reader, expected.chars().count()).map_err(|e| e.to_string())?;
        if actual != expected {
            Err(format!("Expected '{}', found '{}'", expected, actual))
        } else {
            Ok(())
        }
    }

    /// Reads from the stream until the given character is encountered (inclusive).
    fn read_until<R: BufRead>(reader: &mut R, delimiter: char) -> std::io::Result<String> {
        let mut result = String::new();
        let mut total_bytes = 0;

        loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break; // EOF
            }

            let text = std::str::from_utf8(buf).unwrap_or_default();
            let mut chars = text.chars().peekable();

            while let Some(&c) = chars.peek() {
                let len = c.len_utf8();
                result.push(c);
                total_bytes += len;
                chars.next();

                if c == delimiter {
                    reader.consume(total_bytes);
                    return Ok(result);
                }
            }

            reader.consume(total_bytes);
            total_bytes = 0;
        }

        Ok(result)
    }

    /// Reads bytes from `start` (inclusive) to `end` (exclusive) from a seekable buffered reader.
    pub fn read_file<R: BufRead>(reader: &mut R, start: u32, len: u32) -> std::io::Result<Vec<u8>> {
        let mut skipped_chars = 0;
        while skipped_chars < start {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "EOF while skipping",
                ));
            }

            let text = std::str::from_utf8(buf).unwrap_or_default();
            let mut chars = text.chars();

            let mut bytes_to_consume = 0;
            for c in &mut chars {
                bytes_to_consume += c.len_utf8();
                skipped_chars += 1;
                if skipped_chars == start {
                    break;
                }
            }

            reader.consume(bytes_to_consume);
        }

        let mut buf = vec![0u8; len as usize];
        reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Skips the stream until (and including) the first occurrence of `delimiter`.
    pub fn skip_after<R: BufRead>(reader: &mut R, delimiter: char) -> std::io::Result<()> {
        let mut total_bytes = 0;

        loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break; // EOF
            }

            let text = std::str::from_utf8(buf).unwrap_or_default();
            let mut chars = text.chars();

            for c in &mut chars {
                let len = c.len_utf8();
                total_bytes += len;
                if c == delimiter {
                    reader.consume(total_bytes);
                    return Ok(());
                }
            }

            reader.consume(total_bytes);
            total_bytes = 0;
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            format!("Delimiter '{}' not found in stream", delimiter),
        ))
    }
}
