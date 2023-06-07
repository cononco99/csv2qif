use std::collections::HashMap;    
use crate::csv_reading::CsvReading;       
use std::io::{BufRead, Seek};
use crate::find_matching_line::find_matching_line;
use stable_eyre::eyre::*;


pub struct Readers<'a> {
    readers: HashMap<String, &'a dyn CsvReading>,
}

impl<'a> Readers<'a> {
    pub fn new() -> Self {
        Readers { readers : HashMap::new(), }
    }

    pub fn register(&mut self, csv_reading: &'a dyn CsvReading) {
        self.readers.insert(csv_reading.csv_header(), csv_reading);
    }

    pub fn identify_reader<T>(&mut self, buf_reader : &mut T) -> Result<Option<&'a dyn CsvReading>> 
    where T: Seek + BufRead {

        find_matching_line(buf_reader, &self.readers)
    }
}

