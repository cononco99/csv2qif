use chrono::NaiveDate;
use regex::Regex;
use serde::Deserialize;
use stable_eyre::eyre::*;
use std::io::BufRead;
use std::path::PathBuf;
use std::result::Result::Ok;

use crate::csv_reader::*;
use crate::transactions_qif::*;

#[derive(Clone, Copy)]
pub struct SoFiReader;

impl CsvReader for SoFiReader {
    fn csv_header(&self) -> String {
        r#"Date,Description,Type,Amount,Current balance,Status"#
            .to_string()
    }

    // read transactions from qif, then convert to qif_actions.
    // Both of these processes are specific to the particular type of account.
    fn to_transactions(
        &self,
        bufreader: &mut dyn BufRead,
        _current_securities_file: &Option<PathBuf>,
    ) -> Result<Transactions> {
        let sofi_transactions = Self::read_transactions_csv(bufreader)?;
        let sofi_transactions_reversed: Vec<SoFiTransaction> =
            sofi_transactions.iter().rev().cloned().collect(); // we want oldest first

        let from_sofi_transaction = SoFiTransaction::to_qif_action;
        let nested_actions = sofi_transactions_reversed
            .iter()
            .map(from_sofi_transaction)
            .collect::<Result<Vec<_>>>()?;
        let qif_actions = nested_actions.into_iter().flatten().collect();
        Ok(Transactions {
            qif_actions,
            symbols: None,
        })
    }
}

impl SoFiReader {
    fn read_transactions_csv(bufreader: &mut dyn BufRead) -> Result<Vec<SoFiTransaction>> {
        let mut transactions = Vec::new();
        let mut rdr = csv::Reader::from_reader(bufreader);
        let mut should_be_done = false;
        for result in rdr.deserialize() {
            if should_be_done {
                return Err(eyre!(
                    "Still getting transactions csv content when should be done"
                ));
            }
            if let Ok(record) = result {
                // ended up doing this because I could not figure out how to give a type to record
                // If I could have done that, I could have constructed a non mutable cleaned_record.
                let cleaned_record: SoFiTransaction = record;
                transactions.push(cleaned_record);
            } else {
                // sofi has one bad line at end of csv file.
                should_be_done = true;
            }
        }
        Ok(transactions)
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

impl SoFiTransaction {

    fn get_date(&self) -> Result<NaiveDate> {
        let first_try = NaiveDate::parse_from_str(&self.date, "%m/%d/%Y");
        match first_try {
            Ok(successful_date_first_try) => Ok(successful_date_first_try),
            Err(_) => {
                let second_try_re = Regex::new(
                    r"(?x)^
                                   \d{2}/\d{2}/\d{4}      # first date
                                   \ as\ of                 # strike price
                                   \ (\d{2}/\d{2}/\d{4})    # as of date - captured
                                  $",
                )?;

                if let Some(cap) = second_try_re.captures_iter(&self.date).next() {
                    let second_try = NaiveDate::parse_from_str(&cap[1], "%m/%d/%Y");
                    match second_try {
                        Ok(successful_date_second_try) => {
                            return Ok(successful_date_second_try);
                        }
                        Err(_) => {
                            let err_msg = "Could not parse date from sofi on second try: "
                                .to_string()
                                + &self.date;
                            return Err(eyre!(err_msg));
                        }
                    }
                }
                let err_msg = "Could not match date from sofi: ".to_string() + &self.date;
                Err(eyre!(err_msg))
            }
        }
    }

    fn to_qif_action(
        sofi_transaction: &SoFiTransaction,
    ) -> Result<Vec<QifAction>> {
        let mut res: Vec<QifAction> = Vec::new();

        let csv_type = sofi_transaction.transaction_type.as_str();
        match csv_type {
            "Withdrawal" | "Deposit" => {
                res.push(QifAction::LinkedAccountOnly {
                    date: sofi_transaction.get_date()?,
                    payee: sofi_transaction.description.clone(),
                    memo: sofi_transaction.description.clone(),
                    amount: sofi_transaction.amount.clone(),
                });
            }

            _ => {
                println!("Unrecognized action found in .CSV : \"{}\".", csv_type);

                let linked_only = QifAction::LinkedAccountOnly {
                    date: sofi_transaction.get_date()?,
                    payee: sofi_transaction.description.clone(),
                    memo: sofi_transaction.description.clone(),
                    amount: sofi_transaction.amount.clone(),
                };
                println!(
                    "No quantity, price or fees found so entering in linked account only."
                );
                println!("{:#?}", linked_only);

                res.push(linked_only);
            }
        };
        Ok(res)
    }
}
