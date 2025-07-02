use std::io::{Read, Result};

pub fn skip<R: Read>(reader: &mut R, n: u64) -> Result<()> {
    let mut to_skip = n;
    let mut buffer = [0u8; 4096];

    while to_skip > 0 {
        let read_len = std::cmp::min(to_skip, buffer.len() as u64) as usize;
        let bytes_read = reader.read(&mut buffer[..read_len])?;
        if bytes_read == 0 {
            break; // EOF
        }
        to_skip -= bytes_read as u64;
    }

    Ok(())
}

pub fn exact<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<()> {
    reader
        .read_exact(buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::UnexpectedEof, e))
}

pub fn u8<R: Read>(reader: &mut R) -> Result<u8> {
    let mut buf = [0; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

pub fn u32<R: Read>(reader: &mut R) -> Result<u32> {
    let mut buf = [0; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub fn u64<R: Read>(reader: &mut R) -> Result<u64> {
    let mut buf = [0; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

pub fn string<R: Read>(reader: &mut R) -> Result<String> {
    let len = u32(reader)?;
    string_raw(reader, len)
}

pub fn string_raw<R: Read>(reader: &mut R, len: u32) -> Result<String> {
    let len = len as usize;
    let mut buf = vec![0; len];
    reader.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

pub fn array<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let len = u32(reader)?;
    let mut buf = vec![0; len as usize];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}
