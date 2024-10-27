use std::{
    any::Any,
    io::{self, BufReader, Read},
};

use crate::mview_entry::MViewEntry;

const BUFFERED_CAPACITY: usize = 128 * 1024; // 128 KiB

enum InnerReader<R: Read> {
    Direct(R),              // Already buffered by caller
    Buffered(BufReader<R>), // Explicitly buffered by us
}

pub struct MViewFile<R: Read> {
    inner: InnerReader<R>,
}

impl<R: Read + Any> MViewFile<R> {
    pub fn new(reader: R) -> Self {
        // Check if the reader is already a BufReader
        let inner = if (&reader as &dyn Any)
            .downcast_ref::<BufReader<R>>()
            .is_some()
        {
            InnerReader::Direct(reader)
        } else {
            InnerReader::Buffered(BufReader::with_capacity(BUFFERED_CAPACITY, reader))
        };

        Self { inner }
    }
}

impl<R: Read> Iterator for MViewFile<R> {
    type Item = io::Result<MViewEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        let reader: &mut dyn Read = match &mut self.inner {
            InnerReader::Direct(reader) => reader,
            InnerReader::Buffered(reader) => reader,
        };

        match MViewEntry::try_from(&mut reader.bytes()) {
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => None,
            item => Some(item),
        }
    }
}
