use chrono::NaiveDate;
use regex::Regex;
use serde::Deserialize;
use stable_eyre::eyre::*;
use std::fmt::Write as FmtWrite;
use std::path::PathBuf;
use std::result::Result::Ok;
use std::{fs::File, io::BufRead, io::BufReader};

use crate::security::*;
use crate::symbols::*;
use crate::transactions_qif::*;

#[derive(Debug, Clone, Deserialize)]
pub struct SchwabTransaction {
    #[serde(rename = "Date")]
    pub date: String,
    #[serde(rename = "Action")]
    pub action: String,
    #[serde(rename = "Symbol")]
    pub symbol: String,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Quantity")]
    pub quantity: String,
    #[serde(rename = "Price")]
    pub price: String,
    #[serde(rename = "Fees & Comm")]
    pub fees: String,
    #[serde(rename = "Amount")]
    pub amount: String,
}

pub fn read_transactions_csv(filename: &PathBuf) -> Result<Vec<SchwabTransaction>> {
    let file = File::open(filename)?;
    let mut bufreader = BufReader::new(file);
    {
        let mut line = String::new();
        let _ = bufreader.read_line(&mut line)?;
    }
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
            let mut cleaned_record: SchwabTransaction = record;
            cleaned_record.price = cleaned_record.price.replace('$', "");
            cleaned_record.fees = cleaned_record.fees.replace('$', "");
            cleaned_record.amount = cleaned_record.amount.replace('$', "");
            transactions.push(cleaned_record);
        } else {
            // schwab has one bad line at end of csv file.
            should_be_done = true;
        }
    }
    Ok(transactions)
}

impl SchwabTransaction {
    fn get_option(&self) -> Result<(String, String)> {
        let symbol_re = Regex::new(
            r"(?x)^
                               ([A-Z]*)                 # underlying symbol
                               \ (\d{2}/\d{2}/\d{4})    # expiration date
                               \ ([\d\.]*)              # strike price
                               \ ([PC])                 # put or call
                              $",
        )?;

        // symbol and description should both indicate (or not indicate) option.
        // Nested for loops implement check (sort of).
        if let Some(symbol_cap) = symbol_re.captures_iter(self.symbol.as_str()).next() {
            let description_re = Regex::new(
                r"(?x)^
                                   (PUT|CALL)              # PUT or CALL
                                   \ ([^\$]*)\$            # description of underlying
                                   ([\d\.]*)               # strike price
                                   \ EXP                   # EXP
                                   \ (\d{2}/\d{2}/\d{2})   # expiration date
                                   $",
            )?;

            if let Some(description_cap) = description_re
                .captures_iter(self.description.as_str())
                .next()
            {
                let description_strike_price = description_cap[3].parse::<f32>()?;
                let strike_price = &symbol_cap[3];
                let symbol_strike_price = strike_price.parse::<f32>()?;
                assert_eq!(description_strike_price, symbol_strike_price);

                let expiration = NaiveDate::parse_from_str(&description_cap[4], "%m/%d/%y")?;
                let symbol_expiration_date = NaiveDate::parse_from_str(&symbol_cap[2], "%m/%d/%Y")?;
                assert_eq!(expiration, symbol_expiration_date);

                let description_is_call = description_cap[1].eq("CALL");
                let symbol_is_call = symbol_cap[4].eq("C");
                assert_eq!(description_is_call, symbol_is_call);

                let strike_re = Regex::new(
                    r"(?x)^
                                       ([\d]*)                 # dollars
                                       \.
                                       ([\d]*)                 # cents
                                       $",
                )?;

                let mut matched = false;
                let mut strike_string = String::new();
                for strike_cap in strike_re.captures_iter(strike_price) {
                    if matched {
                        return Err(eyre!("got multiple matches on strike"));
                    }
                    matched = true;
                    let dollars = &strike_cap[1];
                    let cents = &strike_cap[2];
                    strike_string =
                        format!("{:0>5}", dollars).to_string() + &format!("{:0<3}", cents);
                }
                if !matched {
                    return Err(eyre!("got no matches on strike"));
                }

                let padded_symbol = format!("{: <6}", &symbol_cap[1]);

                let symbol = padded_symbol
                    + &expiration.format("%y%m%d").to_string()
                    + &symbol_cap[4]
                    + &strike_string;

                let name = description_cap[1].to_string()
                    + " : "
                    + description_cap[2].trim_end()
                    + " - "
                    + &symbol_cap[1]
                    + " "
                    + &expiration.format("%m/%d/%Y").to_string()
                    + " "
                    + strike_price
                    + " "
                    + &symbol_cap[4];

                return Ok((symbol, name));
            }
            // at some point should also add check for quantity * 100
            let mut w = String::new();
            write!(
                &mut w,
                "Symbol {} looks like option but description {} does not.",
                self.symbol, self.description
            )?;
            return Err(eyre!(w));
        }
        Err(eyre!("This is not an option!"))
    }

    pub fn security_details(&self) -> Result<(String, String, SecurityType)> {
        let option_result = SchwabTransaction::get_option(self);
        match option_result {
            Ok((symbol, name)) => Ok((symbol, name, SecurityType::Option)),
            Err(_) => {
                let name = self.description.clone();
                let symbol = self.symbol.clone();
                Ok((symbol, name, SecurityType::Stock))
            }
        }
    }

    pub fn get_date(&self) -> Result<NaiveDate> {
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
                            let err_msg = "Could not parse date from schwab on second try: "
                                .to_string()
                                + &self.date;
                            return Err(eyre!(err_msg));
                        }
                    }
                }
                let err_msg = "Could not match date from schwab: ".to_string() + &self.date;
                Err(eyre!(err_msg))
            }
        }
    }

    pub fn to_transactions(
        schwab_transactions: &[SchwabTransaction],
        current_securities_file: &PathBuf,
    ) -> Result<Transactions> {
        let schwab_transactions_reversed: Vec<SchwabTransaction> =
            schwab_transactions.iter().rev().cloned().collect(); // we want oldest first
        let mut symbols = Symbols::new(current_securities_file)?;

        let from_schwab_transaction = |tr| QifAction::from_schwab_transaction(tr, &mut symbols);
        let nested_actions = schwab_transactions_reversed
            .iter()
            .map(from_schwab_transaction)
            .collect::<Result<Vec<_>>>()?;
        let qif_actions = nested_actions.into_iter().flatten().collect();
        Ok(Transactions {
            qif_actions,
            symbols,
        })
    }
}
