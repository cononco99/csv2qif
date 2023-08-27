use stable_eyre::eyre::*;
use std::io::BufRead;
use std::path::PathBuf;

use crate::transactions_qif::*;
use crate::transaction::*;

pub trait CsvReader {
    fn csv_header(&self) -> String;

    fn to_transactions(
        &self,
        bufreader: &mut dyn BufRead,
    ) -> Result<Vec<Box<dyn Transaction>>> ;
}

impl dyn CsvReader {
    // read transactions from qif, then convert to qif_actions.
    // Both of these processes are specific to the particular type of account.
    pub fn to_qif_transactions(
        &self,
        bufreader: &mut dyn BufRead,
        _current_securities_file: &Option<PathBuf>,
    ) -> Result<QifTransactions> {
            

        let qif_actions = self
            .to_transactions(bufreader)?
            .into_iter()
            .rev() // we want oldest first
            .collect::<Vec<_>>() // this collect/iter pair seems redundant
            .iter()              // how can it be removed ?
            .map(|tr:&Box<dyn Transaction>| (*tr).to_qif_action())
            .collect::<Result<Vec<_>>>()?  // change many Result(s) into one Result
            .into_iter()
            .flatten()
            .collect();

        Ok(QifTransactions {
            qif_actions,
            symbols: None,
        })
    }
}

