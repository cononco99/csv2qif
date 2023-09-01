use stable_eyre::eyre::*;
use std::io::BufRead;

use crate::symbols::*;
use crate::transaction::*;
use crate::transactions_qif::*;

pub trait CsvReader {
    fn csv_header(&self) -> String;

    fn to_transactions(&self, bufreader: &mut dyn BufRead) -> Result<Vec<Box<dyn Transaction>>>;
}

impl dyn CsvReader {
    // read transactions from qif, then convert to qif_actions.
    // Both of these processes are specific to the particular type of account.
    pub fn to_qif_transactions(
        &self,
        bufreader: &mut dyn BufRead,
        securities : &mut Option<Symbols>,
    ) -> Result<QifTransactions> {
        let qif_actions = self
            .to_transactions(bufreader)?
            .into_iter()
            .rev() // we want oldest first
            .map(|tr| tr.to_qif_action(securities ))
            .collect::<Result<Vec<_>>>()? // change many Result(s) into one Result
            .into_iter()
            .flatten()   // to_qif_action may generate multiple qif actions for a transaction
            .collect();

        Ok(QifTransactions {
            qif_actions,
            symbols: securities.take(),
        })
    }
}
