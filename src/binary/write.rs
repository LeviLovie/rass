use std::io::{Result, Write};

pub fn u8<W: Write>(writer: &mut W, value: u8) -> Result<()> {
    writer.write_all(&[value])
}

pub fn u32<W: Write>(writer: &mut W, value: u32) -> Result<()> {
    writer.write_all(&value.to_le_bytes())
}

pub fn u64<W: Write>(writer: &mut W, value: u64) -> Result<()> {
    writer.write_all(&value.to_le_bytes())
}

pub fn string<W: Write>(writer: &mut W, value: &str) -> Result<()> {
    let bytes = value.as_bytes();
    u32(writer, bytes.len() as u32)?;
    writer.write_all(bytes)
}

pub fn string_raw<W: Write>(writer: &mut W, value: &str) -> Result<()> {
    let bytes = value.as_bytes();
    writer.write_all(bytes)
}

pub fn array<W: Write>(writer: &mut W, value: &[u8]) -> Result<()> {
    u32(writer, value.len() as u32)?;
    writer.write_all(value)
}

pub fn array_raw<W: Write>(writer: &mut W, value: &[u8]) -> Result<()> {
    writer.write_all(value)
}
