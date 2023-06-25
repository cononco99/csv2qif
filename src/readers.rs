use crate::csv_reader::CsvReader;
use crate::find_matching_line::find_matching_line;
use stable_eyre::eyre::*;
use std::collections::HashMap;
use std::io::{BufRead, Seek};

pub struct Readers<'a> {
    readers: HashMap<String, &'a dyn CsvReader>,
}

impl<'a> Readers<'a> {
    pub fn new() -> Self {
        Readers {
            readers: HashMap::new(),
        }
    }

    pub fn register(&mut self, csv_reader: &'a dyn CsvReader) {
        self.readers.insert(csv_reader.csv_header(), csv_reader);
    }

    pub fn identify_reader<T>(&mut self, buf_reader: &mut T) -> Result<Option<&'a dyn CsvReader>>
    where
        T: Seek + BufRead,
    {
        find_matching_line(buf_reader, &self.readers)
    }
}
