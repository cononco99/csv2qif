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
        let transactions = self.to_transactions(bufreader)?;
        let transactions_reversed: Vec<_> =
            transactions.into_iter().rev().collect(); // we want oldest first

        let from_transaction = |tr:&Box<dyn Transaction>| (*tr).to_qif_action();
        let nested_actions = transactions_reversed
            .iter()
            .map(from_transaction)
            .collect::<Result<Vec<_>>>()?;
        let qif_actions = nested_actions.into_iter().flatten().collect();
        Ok(QifTransactions {
            qif_actions,
            symbols: None,
        })
    }
}

