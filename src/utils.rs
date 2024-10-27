use std::io;

pub fn read_cstr<R: io::Read>(reader: &mut io::Bytes<R>) -> io::Result<String> {
    let bytes = reader
        .take_while(|byte| byte.as_ref().map_or(false, |b| *b != b'\0'))
        .collect::<Result<_, _>>()?;
    String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn read_u32<R: io::Read>(reader: &mut io::Bytes<R>) -> io::Result<u32> {
    Ok(u32::from_le_bytes(
        reader
            .take(4)
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected end of file"))?,
    ))
}

pub fn next_or_eof<R: io::Read>(reader: &mut io::Bytes<R>) -> io::Result<u8> {
    reader
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected end of file"))?
}
