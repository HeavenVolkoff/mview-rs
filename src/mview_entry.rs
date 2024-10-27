use std::{cmp::Ordering, io};

use crate::utils::{next_or_eof, read_cstr, read_u32};

const MAX_SEQUENCES: usize = 4096;
const INITIAL_SEQUENCE: usize = 256;

#[derive(Debug, Clone)]
struct Sequence {
    start: usize,
    length: usize,
}

impl Default for Sequence {
    fn default() -> Self {
        Self {
            start: 0,
            length: 1,
        }
    }
}

#[derive(Debug)]
pub struct MViewEntry {
    pub name: String,
    pub data: Vec<u8>,
    pub mime_type: String,
}

impl MViewEntry {
    /// Decompresses data using a custom LZW-like algorithm
    fn decompress<R: io::Read>(
        data: &mut io::Bytes<R>,
        compressed_size: usize,
        uncompressed_size: usize,
    ) -> io::Result<Vec<u8>> {
        let mut decompressed = vec![0; uncompressed_size];
        decompressed[0] = next_or_eof(data)?;

        let mut sequence = INITIAL_SEQUENCE;
        let mut previous = Sequence::default();
        let mut sequences = vec![Sequence::default(); MAX_SEQUENCES].into_boxed_slice();

        let mut index = 1;
        let mut counter = 1;
        let mut prev_high = u16::from(next_or_eof(data)?);
        loop {
            // Calculate bit-packed position
            let packed_pos = 1 + counter + counter / 2;
            if packed_pos >= compressed_size {
                break;
            }

            // Extract 12-bit sequence code from packed data
            let code = if counter % 2 == 0 {
                let low = u16::from(next_or_eof(data)?);
                let high = u16::from(next_or_eof(data)?);
                prev_high = high;
                ((high & 15) << 8) | low
            } else {
                let low = prev_high;
                let high = u16::from(next_or_eof(data)?);
                high << 4 | low >> 4
            } as usize;

            let (start, length) = match code.cmp(&sequence) {
                Ordering::Less => {
                    // Code exists in sequence list
                    if code < INITIAL_SEQUENCE {
                        // Code represents a single ASCII character
                        let start = index;

                        decompressed[start] = u8::try_from(code)
                            .expect("This must never fail due to the previous condition");
                        index += 1;

                        (start, 1)
                    } else {
                        // Code represents an index to a previously decoded sequence of bytes
                        let start = sequences[code].start;
                        let length = sequences[code].length;

                        let (left, right) = decompressed.split_at_mut(index);
                        right[..length].copy_from_slice(&left[start..start + length]);

                        index += length;

                        (start, length)
                    }
                }
                Ordering::Equal => {
                    // Special case: repeat previous sequence data + its first byte
                    let start = index;

                    let (left, right) = decompressed.split_at_mut(start);
                    right[..previous.length]
                        .copy_from_slice(&left[previous.start..previous.start + previous.length]);

                    index += previous.length;
                    decompressed[index] = decompressed[previous.start];
                    index += 1;

                    (start, previous.length + 1)
                }
                Ordering::Greater => {
                    // Invalid code, discard remaining data
                    data.take(compressed_size - packed_pos).for_each(drop);
                    break;
                }
            };

            // Update list with new sequence
            sequences[sequence].start = previous.start;
            sequences[sequence].length = previous.length + 1;

            // Save current sequence for next iteration
            previous.start = start;
            previous.length = length;

            // Update sequence counter
            sequence += 1;
            if sequence >= MAX_SEQUENCES {
                sequence = INITIAL_SEQUENCE;
            }

            counter += 1;
        }

        Ok(decompressed)
    }
}

impl<R: io::Read> TryFrom<&mut io::Bytes<R>> for MViewEntry {
    type Error = io::Error;

    fn try_from(bytes: &mut io::Bytes<R>) -> Result<Self, Self::Error> {
        let name = read_cstr(bytes)?;
        let mime_type = read_cstr(bytes)?;
        let is_compressed = (read_u32(bytes)? & 1) != 0;
        let data_size = read_u32(bytes)? as usize;
        let uncompressed_size = read_u32(bytes)? as usize;

        let data = if is_compressed {
            Self::decompress(bytes, data_size, uncompressed_size)
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
