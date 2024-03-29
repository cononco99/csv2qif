use chrono::NaiveDate;
use regex::Regex;
use serde::Deserialize;
use stable_eyre::eyre::*;
use std::fmt::Write as FmtWrite;
use std::io::BufRead;
use std::result::Result::Ok;

use crate::csv_reader::*;
use crate::security::SecurityType;
use crate::symbols::Symbols;
use crate::transaction::*;
use crate::transactions_qif::*;

pub struct SchwabReader;

impl Reader for SchwabReader {
    fn csv_header(&self) -> String {
        r#""Date","Action","Symbol","Description","Price","Quantity","Fees & Comm","Amount""#
            .to_string()
    }

    fn to_transactions(
        &self,
        bufreader: &mut dyn BufRead,
        securities: &mut Option<Symbols>,
    ) -> Result<Vec<QifAction>> {
        <dyn Reader>::from_csv::<SchwabTransaction>(bufreader, securities)
    }
}

pub struct SchwabReaderOldCsv;

impl Reader for SchwabReaderOldCsv {
    fn csv_header(&self) -> String {
        r#""Date","Action","Symbol","Description","Quantity","Price","Fees & Comm","Amount""#
            .to_string()
    }

    fn to_transactions(
        &self,
        bufreader: &mut dyn BufRead,
        securities: &mut Option<Symbols>,
    ) -> Result<Vec<QifAction>> {
        <dyn Reader>::from_csv::<SchwabTransaction>(bufreader, securities)
    }
}

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

impl SchwabTransaction {
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
}

impl Transaction for SchwabTransaction {
    fn to_qif_action(&self, opt_symbols: &mut Option<Symbols>) -> Result<Vec<QifAction>> {
        let mut cleaned_record: SchwabTransaction = self.clone();

        // remove dollar signs that schwab puts into csv
        // use cleaned_record instead of self starting here.
        // may be more cleanups that could be moved here but be sure to test....
        cleaned_record.price = cleaned_record.price.replace('$', "");
        cleaned_record.fees = cleaned_record.fees.replace('$', "");
        cleaned_record.amount = cleaned_record.amount.replace('$', "");

        let symbols = opt_symbols
            .as_mut()
            .ok_or(eyre!("Expected symbols but none provided."))?;
        let mut res: Vec<QifAction> = Vec::new();

        let csv_action = cleaned_record.action.as_str();
        match csv_action {
            "Sell to Open" => {
                let trade = Self::to_trade(&cleaned_record, symbols)?;
                res.push(QifAction::ShtSell { trade })
            }
            "Buy to Close" => {
                let trade = Self::to_trade(&cleaned_record, symbols)?;
                res.push(QifAction::CvrShrt { trade })
            }
            "Buy" | "Buy to Open" => {
                let trade = Self::to_trade(&cleaned_record, symbols)?;
                res.push(QifAction::Buy { trade });
            }
            "Sell" | "Sell to Close" => {
                let trade = Self::to_trade(&cleaned_record, symbols)?;
                res.push(QifAction::Sell { trade });
            }
            "Expired" => {
                let trade = Self::to_expired_transaction(&cleaned_record, symbols)?;
                res.push(QifAction::Sell { trade });
            }
            "Margin Interest" => {
                // Margin Interest from schwab is negative but quicken wants it positive.
                // Hence the trim_start_matches hack for amount
                res.push(QifAction::MargInt {
                    date: cleaned_record.get_date()?,
                    memo: cleaned_record.description.clone(),
                    amount: cleaned_record.amount.trim_start_matches('-').to_string(),
                });
            }
            "Pr Yr Special Div" | "Cash Dividend" => {
                let (symbol, name, security_type) = cleaned_record.security_details()?;
                symbols.enter_if_not_found(&symbol, &name, &security_type)?;
                res.push(QifAction::Div {
                    date: cleaned_record.get_date()?,
                    symbol,
                    amount: cleaned_record.amount.clone(),
                });
            }
            "Qualified Dividend" => {
                let (symbol, name, security_type) = cleaned_record.security_details()?;
                symbols.enter_if_not_found(&symbol, &name, &security_type)?;
                res.push(QifAction::Div {
                    date: cleaned_record.get_date()?,
                    symbol,
                    amount: cleaned_record.amount.clone(),
                });
            }
            "Short Term Cap Gain" => {
                let (symbol, name, security_type) = cleaned_record.security_details()?;
                symbols.enter_if_not_found(&symbol, &name, &security_type)?;
                res.push(QifAction::CGShort {
                    date: cleaned_record.get_date()?,
                    symbol,
                    amount: cleaned_record.amount.clone(),
                });
            }
            "Long Term Cap Gain" => {
                let (symbol, name, security_type) = cleaned_record.security_details()?;
                symbols.enter_if_not_found(&symbol, &name, &security_type)?;
                res.push(QifAction::CGLong {
                    date: cleaned_record.get_date()?,
                    symbol,
                    amount: cleaned_record.amount.clone(),
                });
            }
            "Foreign Tax Paid" | "ADR Mgmt Fee" | "Cash In Lieu" | "MoneyLink Deposit"
            | "Wire Funds" | "Misc Cash Entry" | "Service Fee" | "Journal"
            | "MoneyLink Transfer" | "Pr Yr Cash Div" | "Pr Yr Cash Div Adj" | "Bank Interest"
            | "Credit Interest" | "Funds Paid" => {
                res.push(QifAction::Generic {
                    date: cleaned_record.get_date()?,
                    payee: cleaned_record.description.clone(),
                    memo: Some(cleaned_record.description.clone()),
                    category: None,
                    amount: cleaned_record.amount.clone(),
                });
            }

            "Spin-off" => {
                let (symbol, name, security_type) = cleaned_record.security_details()?;
                let quantity = cleaned_record.quantity.parse::<i32>()?;
                let date: NaiveDate = cleaned_record.get_date()?;
                symbols.enter_if_not_found(&symbol, &name, &security_type)?;
                res.push(QifAction::ShrsIn {
                    date,
                    symbol,
                    quantity,
                });
            }

            "Stock Split" => {
                println!("Stock Split not handled.");
                println!("This is because Schwab CSV contains the number of new shared added due to the split but quicken records the factor that the old number of shared is multiplied by to get the new number of shares.  Without knowing the starting number of shares, the factor can not be calculated.  The split will have to be entered by hand:");
                println!("{:#?}", cleaned_record);
                println!();
            }

            "Journaled Shares" => {
                println!("Journaled Shares not handled.");
                println!("Journaled Shares indicates a transfer of shares from one account to another.   This transfer will have to be entered by hand:");
                println!("{:#?}", cleaned_record);
                println!();
            }

            "Name Change" => {
                println!("Name change not handled:");
                println!("{:#?}", cleaned_record);
                println!();
            }

            _ => {
                if (cleaned_record.quantity.is_empty())
                    && (cleaned_record.price.is_empty())
                    && (cleaned_record.fees.is_empty())
                {
                    println!("Unrecognized action found in .CSV : \"{}\".", csv_action);

                    let generic = QifAction::Generic {
                        date: cleaned_record.get_date()?,
                        payee: cleaned_record.description.clone(),
                        memo: Some(cleaned_record.description.clone()),
                        category: None,
                        amount: cleaned_record.amount.clone(),
                    };
                    println!("No quantity, price or fees found so entering as cash transaction.");
                    println!("{:#?}", generic);

                    res.push(generic);
                } else {
                    let message =
                        "Unrecognized action found in .CSV file : ".to_string() + csv_action;
                    return Err(eyre!(message));
                }
            }
        };
        Ok(res)
    }
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

    fn security_details(&self) -> Result<(String, String, SecurityType)> {
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

    fn to_trade(schwab_transaction: &SchwabTransaction, symbols: &mut Symbols) -> Result<Trade> {
        let (symbol, name, security_type) = schwab_transaction.security_details()?;

        let price = schwab_transaction.price.to_string();
        let quantity = schwab_transaction.quantity.to_string()
            + if security_type == SecurityType::Option {
                "00"
            } else {
                ""
            };
        let amount = schwab_transaction
            .amount
            .trim_start_matches('-')
            .to_string();
        let fees = schwab_transaction.fees.to_string();
        let date: NaiveDate = schwab_transaction.get_date()?;
        symbols.enter_if_not_found(&symbol, &name, &security_type)?;
        let res = Trade {
            date,
            symbol,
            price,
            quantity,
            amount,
            fees,
        };
        Ok(res)
    }

    fn to_expired_transaction(
        schwab_transaction: &SchwabTransaction,
        symbols: &mut Symbols,
    ) -> Result<Trade> {
        let price = "".to_string();
        let amount = "".to_string();
        let fees = "".to_string();

        let (symbol, name, security_type) = schwab_transaction.security_details()?;
        if security_type != SecurityType::Option {
            return Err(eyre!("Expired found in CSV for non-option"));
        }

        // negation hack
        let (leading, rest) = schwab_transaction.quantity.split_at(1);
        let q = if leading == "-" {
            rest.to_string()
        } else {
            "-".to_string() + &schwab_transaction.quantity
        };

        let quantity = q + "00";
        let date: NaiveDate = schwab_transaction.get_date()?;
        symbols.enter_if_not_found(&symbol, &name, &security_type)?;
        let res = Trade {
            date,
            symbol,
            price,
            quantity,
            amount,
            fees,
        };
        Ok(res)
    }
}
