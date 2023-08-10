use stable_eyre::eyre::*;
use std::io::BufRead;
use std::path::PathBuf;

use crate::transactions_qif::*;

pub trait CsvReader {
    fn csv_header(&self) -> String;

    fn to_transactions(
        &self,
        bufreader: &mut dyn BufRead,
        current_securities_file: &Option<PathBuf>,
    ) -> Result<Transactions>;
}
