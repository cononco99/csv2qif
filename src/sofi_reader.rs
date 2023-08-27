use chrono::NaiveDate;
use serde::Deserialize;
use stable_eyre::eyre::*;
use std::io::BufRead;
use std::path::PathBuf;
use std::result::Result::Ok;

use crate::csv_reader::*;
use crate::transaction::*;
use crate::transactions_qif::*;


#[derive(Clone, Copy)]
pub struct SoFiReader;

impl CsvReader for SoFiReader {
    fn csv_header(&self) -> String {
        r#"Date,Description,Type,Amount,Current balance,Status"#.to_string()
    }

    // read transactions from qif, then convert to qif_actions.
    // Both of these processes are specific to the particular type of account.
    fn to_transactions(
        &self,
        bufreader: &mut dyn BufRead,
        _current_securities_file: &Option<PathBuf>,
    ) -> Result<QifTransactions> {
        let mut transactions : Vec<Box<dyn Transaction>> = Vec::new();
        let mut rdr = csv::Reader::from_reader(bufreader);
        // turbofish applied to function deserialize
        for result in rdr.deserialize::<SoFiTransaction>() {
            transactions.push(Box::new(result?));
        }
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

#[derive(Debug, Clone, Deserialize)]
pub struct SoFiTransaction {
    #[serde(rename = "Date")]
    pub date: String,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Type")]
    pub transaction_type: String,
    #[serde(rename = "Amount")]
    pub amount: String,
    #[serde(rename = "Current balance")]
    pub current_balance: String,
    #[serde(rename = "Status")]
    pub status: String,
}

impl Transaction for SoFiTransaction {
    fn get_date(&self) -> Result<NaiveDate> {
        Ok(NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")?)
    }

    fn to_qif_action(&self) -> Result<Vec<QifAction>> {
        let mut res: Vec<QifAction> = Vec::new();

        let csv_type = self.transaction_type.as_str();
        res.push(QifAction::Generic {
            date: self.get_date()?,
            payee: self.description.clone(),
            memo: None,
            category: Some(self.transaction_type.clone()),
            amount: self.amount.clone(),
        });
        match csv_type {
            "Withdrawal" | "Deposit" => {}

            _ => {
                println!("Unrecognized action found in .CSV : \"{}\".", csv_type);

                println!("No quantity, price or fees found so entering in linked account only.");
            }
        };
        Ok(res)
    }
}

