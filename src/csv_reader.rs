use stable_eyre::eyre::*;
use std::io::BufRead;

use crate::symbols::*;
use crate::transaction::*;
use crate::transactions_qif::*;

pub trait CsvReader {
    fn csv_header(&self) -> String;

    fn to_transactions(&self, bufreader: &mut dyn BufRead, securities: &mut Option<Symbols>) -> Result<Vec<QifAction>>;
}

impl dyn CsvReader {
    // read transactions from qif, then convert to qif_actions.
    // Both of these processes are specific to the particular type of account.
    pub fn to_qif_transactions(
        &self,
        bufreader: &mut dyn BufRead,
        securities: &mut Option<Symbols>,
    ) -> Result<QifTransactions> {
        Ok(QifTransactions {
            qif_actions : self.to_transactions(bufreader, securities)?,
            symbols: securities.take(),
        })
    }



    // Note:  per : https://codeandbitters.com/static-trait-bound/
    //
    // The way that I often think about the 'static trait bound is:
    // "I don't want my generic type T to permit reference types."
    pub fn from_csv<T>(bufreader: &mut dyn BufRead, securities: &mut Option<Symbols>) -> Result<Vec<QifAction>>
    where for<'de> T: serde::Deserialize<'de> + Transaction + 'static,
    {
        let qif_actions = csv::Reader::from_reader(bufreader)
            .deserialize::<T>()              // deserialize to some kind of Transaction
            .collect::<Result<Vec<_>,_>>()?  // collect to make sure they all worked.
            .into_iter()
            .map(|sft| Box::new(sft) as Box<dyn Transaction>)   // put in Box as dyn Transaction
            .rev()                           // we want oldest first
            .map(|transaction| transaction.to_qif_action(securities))
            .collect::<Result<Vec<_>,_>>()?  // collect to make sure they all worked.
            .into_iter()
            .flatten()                       // to_qif_action may generate multiple qif actions 
            .collect();

        Ok(qif_actions)
    }
}
