use std::{
    cmp::Ordering,
    convert::TryInto,
    io::{self, Bytes, Read},
};

const POS_CACHE_SIZE: usize = 4096;
const INITIAL_CODE: usize = 256;
const BYTE_MASK: u16 = 15;

fn read_cstr<R: Read>(reader: &mut Bytes<R>) -> io::Result<String> {
    let bytes = reader
        .take_while(|byte| byte.as_ref().map_or(false, |b| *b != b'\0'))
        .collect::<Result<_, _>>()?;
    String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn read_u32<R: Read>(reader: &mut Bytes<R>) -> io::Result<u32> {
    Ok(u32::from_le_bytes(
        reader
            .take(4)
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected end of file"))?,
    ))
}

fn next_or_eof<R: Read>(reader: &mut Bytes<R>) -> io::Result<u8> {
    reader
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected end of file"))?
}

#[derive(Debug)]
pub struct MViewEntry {
    name: String,
    data: Vec<u8>,
    mime_type: String,
}

impl MViewEntry {
    fn decompress<R: Read>(
        data: &mut Bytes<R>,
        size: usize,
        compressed_size: usize,
    ) -> io::Result<Vec<u8>> {
        let mut decompressed = vec![0; size];
        decompressed[0] = next_or_eof(data)?;

        let mut starts = [0; POS_CACHE_SIZE];
        let mut lengths = [0; POS_CACHE_SIZE];

        let mut prev_start = 0;
        let mut prev_length = 1;

        let mut next_code = INITIAL_CODE;

        let mut index = 1;
        let mut counter = 1;

        let mut prev_high = u16::from(next_or_eof(data)?);
        loop {
            let read_count = 1 + counter + counter / 2;
            if read_count >= compressed_size {
                break;
            }

            let low = if counter % 2 == 0 {
                u16::from(next_or_eof(data)?)
            } else {
                prev_high
            };
            let high = u16::from(next_or_eof(data)?);
            prev_high = high;

            let code = if counter % 2 == 0 {
                (high & BYTE_MASK) << 8 | low
            } else {
                high << 4 | low >> 4
            } as usize;

            let (start, length) = match code.cmp(&next_code) {
                Ordering::Less => {
                    if code < INITIAL_CODE {
                        let start = index;

                        decompressed[start] = u8::try_from(code)
                            .expect("This must never fail due to the previous condition");
                        index += 1;

                        (start, 1)
                    } else {
                        let start = starts[code];
                        let length = lengths[code];
                        let data = decompressed[start..start + length].to_vec();

                        decompressed[index..index + length].copy_from_slice(&data);
                        index += length;

                        (start, length)
                    }
                }
                Ordering::Equal => {
                    let data = decompressed[prev_start..prev_start + prev_length].to_vec();
                    let start = index;

                    decompressed[start..start + prev_length].copy_from_slice(&data);
                    index += prev_length;
                    decompressed[index] = decompressed[prev_start];
                    index += 1;

                    (start, prev_length + 1)
                }
                Ordering::Greater => {
                    data.take(compressed_size - read_count).for_each(drop);
                    break;
                }
            };

            starts[next_code] = prev_start;
            lengths[next_code] = prev_length + 1;

            prev_start = start;
            prev_length = length;

            next_code += 1;
            if next_code >= POS_CACHE_SIZE {
                next_code = INITIAL_CODE;
            }

            counter += 1;
        }

        Ok(decompressed)
    }
}

impl<R: Read> TryFrom<&mut Bytes<R>> for MViewEntry {
    type Error = io::Error;

    fn try_from(bytes: &mut Bytes<R>) -> Result<Self, Self::Error> {
        let name = read_cstr(bytes)?;
        let mime_type = read_cstr(bytes)?;
        let is_compressed = (read_u32(bytes)? & 1) != 0;
        let data_size = read_u32(bytes)? as usize;
        let decompressed_size = read_u32(bytes)? as usize;

        let data = if is_compressed {
            Self::decompress(bytes, decompressed_size, data_size)
        } else {
            bytes.take(data_size).collect::<Result<Vec<_>, _>>()
        }?;

        Ok(Self {
            name,
            data,
            mime_type,
        })
    }
}

pub struct MViewFile<R: Read> {
    reader: R,
}

impl<R: Read> Iterator for MViewFile<R> {
    type Item = io::Result<MViewEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        match MViewEntry::try_from(&mut self.reader.by_ref().bytes()) {
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => None,
            item => Some(item),
        }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let file = std::fs::File::open(filename)?;
    let mview_file = MViewFile { reader: file };

    for entry in mview_file {
        match entry {
            Ok(entry) => {
                println!("Name: {}, MIME Type: {}", entry.name, entry.mime_type);
                // Process the data as needed
                if std::path::Path::new(&entry.name).file_stem()
                    == Some(std::ffi::OsStr::new("thumbnail"))
                {
                    std::fs::write(&entry.name, &entry.data)?;
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        }
    }

    println!("COMPLETED!!!");
    Ok(())
}
