use chrono::NaiveDate;
use serde::Deserialize;
use stable_eyre::eyre::*;
use std::io::BufRead;
use std::result::Result::Ok;

use crate::csv_reader::*;
use crate::transaction::*;
use crate::transactions_qif::*;
use crate::symbols::Symbols;

#[derive(Clone, Copy)]
pub struct SoFiReader;

impl CsvReader for SoFiReader {
    fn csv_header(&self) -> String {
        r#"Date,Description,Type,Amount,Current balance,Status"#.to_string()
    }

    fn to_transactions(&self, bufreader: &mut dyn BufRead) -> Result<Vec<Box<dyn Transaction>>> {
        <dyn CsvReader>::from_csv::<SoFiTransaction>(self, bufreader)
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

    fn to_qif_action(&self, _securities: &mut Option<Symbols>) -> Result<Vec<QifAction>> {
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
