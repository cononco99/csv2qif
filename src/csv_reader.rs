use stable_eyre::eyre::*;
use std::io::BufRead;

use crate::opt::AccountType;
use crate::symbols::*;
use crate::transaction::*;
use crate::transactions_qif::*;

pub trait Reader {
    fn csv_header(&self) -> String;

    fn to_transactions(
        &self,
        bufreader: &mut dyn BufRead,
        securities: &mut Option<Symbols>,
    ) -> Result<Vec<QifAction>>;
}

impl dyn Reader {
    // read transactions from qif, then convert to qif_actions.
    // Both of these processes are specific to the particular type of account.
    pub fn to_qif_transactions(
        &self,
        bufreader: &mut dyn BufRead,
        account_type: AccountType,
        securities: &mut Option<Symbols>,
    ) -> Result<QifTransactions> {
        Ok(QifTransactions {
            qif_actions: self.to_transactions(bufreader, securities)?,
            account_type,
            symbols: securities.take(),
        })
    }

    // Note:  per : https://codeandbitters.com/static-trait-bound/
    //
    // The way that I often think about the 'static trait bound is:
    // "I don't want my generic type T to permit reference types."
    pub fn from_csv<T>(
        bufreader: &mut dyn BufRead,
        securities: &mut Option<Symbols>,
    ) -> Result<Vec<QifAction>>
    where
        for<'de> T: serde::Deserialize<'de> + Transaction + 'static,
    {
        let mut qif_actions: Vec<Vec<QifAction>> = Vec::new();
        let mut rdr = csv::Reader::from_reader(bufreader);
        for record in rdr.deserialize::<T>() {
            if record.is_err() {
                // some csv files are not too clean.
                break;
            }
            let qif_action = record?.to_qif_action(securities)?;
            qif_actions.push(qif_action);
        }

        // reversing because csv files typically have newest transactions first.
        Ok(qif_actions.into_iter().rev().flatten().collect())
    }
}
