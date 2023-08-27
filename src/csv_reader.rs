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
        let from_transaction = |tr:&Box<dyn Transaction>| (*tr).to_qif_action();
            

        let qif_actions = self
            .to_transactions(bufreader)?
            .into_iter()
            .rev()
            .collect::<Vec<_>>() // we want oldest first
            .iter()
            .map(from_transaction)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();

        Ok(QifTransactions {
            qif_actions,
            symbols: None,
        })
    }
}

