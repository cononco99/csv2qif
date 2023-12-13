use chrono::{Datelike, NaiveDate};
use stable_eyre::eyre::*;
use std::fs::File;
use std::io::Write as IoWrite;
use std::path::PathBuf;

use crate::file_names::FileNames;
use crate::security::SecurityType;
use crate::symbols::Symbols;

#[derive(Debug)]
pub struct Trade {
    pub date: NaiveDate,
    pub symbol: String,
    pub price: String,
    pub quantity: String,
    pub amount: String,
    pub fees: String,
}

impl Trade {
    pub fn print(
        &self,
        output: &mut dyn IoWrite,
        action_type: &String,
        cash_account: &Option<String>,
        symbols: Option<&Symbols>,
    ) -> Result<()> {
        let memo = symbols.unwrap().lookup(&self.symbol)?;

        writeln!(
            output,
            "D{}/{}'{}",
            self.date.month(),
            self.date.day(),
            self.date.year() % 100
        )?;
        write!(output, "N{}", action_type)?;
        if cash_account.is_some() {
            write!(output, "X")?;
        }
        writeln!(output)?;
        writeln!(output, "Y{}", memo)?;
        writeln!(output, "I{}", self.price)?;
        writeln!(output, "Q{}", self.quantity)?;
        writeln!(output, "U{}", self.amount)?;
        writeln!(output, "T{}", self.amount)?;
        writeln!(output, "M{}", memo)?;
        writeln!(output, "O{}", self.fees)?;
        if let Some(acctname) = cash_account {
            writeln!(output, "L[{}]", acctname)?
        }
        writeln!(output, "${}", self.amount)?;
        writeln!(output, "^")?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum QifAction {
    ShtSell {
        trade: Trade,
    },
    CvrShrt {
        trade: Trade,
    },
    Buy {
        trade: Trade,
    },
    Sell {
        trade: Trade,
    },
    MargInt {
        date: NaiveDate,
        memo: String,
        amount: String,
    },
    Div {
        date: NaiveDate,
        symbol: String,
        amount: String,
    },
    CGLong {
        date: NaiveDate,
        symbol: String,
        amount: String,
    },
    CGShort {
        date: NaiveDate,
        symbol: String,
        amount: String,
    },
    ShrsIn {
        date: NaiveDate,
        symbol: String,
        quantity: i32,
    },
    Generic {
        date: NaiveDate,
        payee: String,
        memo: Option<String>,
        category: Option<String>,
        amount: String,
    }, //fake
}

impl QifAction {
    pub fn print_transaction(
        &self,
        output: &mut dyn IoWrite,
        cash_account: &Option<String>,
        symbols: Option<&Symbols>,
    ) -> Result<()> {
        match self {
            Self::ShtSell { trade } => {
                trade.print(output, &"ShtSell".to_string(), cash_account, symbols)
            }
            Self::CvrShrt { trade } => {
                trade.print(output, &"CvrShrt".to_string(), cash_account, symbols)
            }
            Self::Buy { trade } => trade.print(output, &"Buy".to_string(), cash_account, symbols),
            Self::Sell { trade } => {
                trade.print(output, &"Sell".to_string(), cash_account, symbols)
            }
            Self::MargInt { date, memo, amount } => {
                writeln!(
                    output,
                    "D{}/{}'{}",
                    date.month(),
                    date.day(),
                    date.year() % 100
                )?;
                write!(output, "NMargInt")?;
                if cash_account.is_some() {
                    write!(output, "X")?;
                }
                writeln!(output)?;
                writeln!(output, "U{}", amount)?;
                writeln!(output, "T{}", amount)?;
                writeln!(output, "M{}", memo)?;
                if let Some(acctname) = cash_account {
                    writeln!(output, "L[{}]", acctname)?
                }
                writeln!(output, "${}", amount)?;
                writeln!(output, "^")?;
                Ok(())
            }
            Self::Generic {
                date,
                payee,
                memo,
                category,
                amount,
            } => {
                writeln!(
                    output,
                    "D{}/{}'{}",
                    date.month(),
                    date.day(),
                    date.year() % 100
                )?;
                writeln!(output, "U{}", amount)?;
                writeln!(output, "T{}", amount)?;
                writeln!(output, "P{}", payee)?;
                if let Some(memo) = memo {
                    writeln!(output, "M{}", memo)?;
                }
                if let Some(category) = category {
                    writeln!(output, "L{}", category)?;
                }
                writeln!(output, "^")?;
                Ok(())
            }
            Self::Div {
                date,
                symbol,
                amount,
            } => {
                let name = symbols.unwrap().lookup(symbol)?;
                writeln!(
                    output,
                    "D{}/{}'{}",
                    date.month(),
                    date.day(),
                    date.year() % 100
                )?;
                write!(output, "NDiv")?;
                if cash_account.is_some() {
                    write!(output, "X")?;
                }
                writeln!(output)?;
                writeln!(output, "Y{}", name)?;
                writeln!(output, "U{}", amount)?;
                writeln!(output, "T{}", amount)?;
                writeln!(output, "M{}", name)?;
                if let Some(acctname) = cash_account {
                    writeln!(output, "L[{}]", acctname)?
                }
                writeln!(output, "${}", amount)?;
                writeln!(output, "^")?;
                Ok(())
            }
            Self::CGLong {
                date,
                symbol,
                amount,
            } => {
                let name = symbols.unwrap().lookup(symbol)?;
                writeln!(
                    output,
                    "D{}/{}'{}",
                    date.month(),
                    date.day(),
                    date.year() % 100
                )?;
                write!(output, "NCGLong")?;
                if cash_account.is_some() {
                    write!(output, "X")?;
                }
                writeln!(output)?;
                writeln!(output, "Y{}", name)?;
                writeln!(output, "U{}", amount)?;
                writeln!(output, "T{}", amount)?;
                writeln!(output, "M{}", name)?;
                if let Some(acctname) = cash_account {
                    writeln!(output, "L[{}]", acctname)?
                }
                writeln!(output, "${}", amount)?;
                writeln!(output, "^")?;
                Ok(())
            }
            Self::CGShort {
                date,
                symbol,
                amount,
            } => {
                let name = symbols.unwrap().lookup(symbol)?;
                writeln!(
                    output,
                    "D{}/{}'{}",
                    date.month(),
                    date.day(),
                    date.year() % 100
                )?;
                write!(output, "NCGShort")?;
                if cash_account.is_some() {
                    write!(output, "X")?;
                }
                writeln!(output)?;
                writeln!(output, "Y{}", name)?;
                writeln!(output, "U{}", amount)?;
                writeln!(output, "T{}", amount)?;
                writeln!(output, "M{}", name)?;
                if let Some(acctname) = cash_account {
                    writeln!(output, "L[{}]", acctname)?
                }
                writeln!(output, "${}", amount)?;
                writeln!(output, "^")?;
                Ok(())
            }
            Self::ShrsIn {
                date,
                symbol,
                quantity,
            } => {
                let name = symbols.unwrap().lookup(symbol)?;
                writeln!(
                    output,
                    "D{}/{}'{}",
                    date.month(),
                    date.day(),
                    date.year() % 100
                )?;
                writeln!(output, "NShrsIn")?;
                writeln!(output, "Y{}", name)?;
                writeln!(output, "Q{}", quantity)?;
                writeln!(output, "M{}", name)?;
                writeln!(output, "^")?;
                Ok(())
            }
        }
    }

    fn cash_only(qa: &&Self) -> bool {
        matches!(
            qa,
            Self::Generic {
                date: _,
                payee: _,
                memo: _,
                category: _,
                amount: _,
            }
        )
    }

    fn not_cash_only(qa: &&Self) -> bool {
        !Self::cash_only(qa)
    }
}

pub struct QifTransactions {
    pub qif_actions: Vec<QifAction>,
    pub symbols: Option<Symbols>,
}

impl QifTransactions {
    pub fn print_transactions_qif(
        &self,
        output_file: &PathBuf,
        cash_account: &Option<String>,
    ) -> Result<()> {
        let invest_transactions = if cash_account.is_some() {
            self
                .qif_actions
                .iter()
                .filter(QifAction::not_cash_only)
                .collect::<Vec<_>>()
        } else {
            self
                .qif_actions
                .iter()
                .collect::<Vec<_>>()
        };

        let transaction_count = invest_transactions.len();

        if transaction_count == 0 {
        } else {
            println!("{} transaction(s) found.", transaction_count);
            // let output_file_str = output_file_str_result.map_err(|e| Err("bad file name"));
            println!(
                "Creating .qif file for these transactions: {} .",
                output_file.as_path().display()
            );
            println!("Import this file into the investment account");
            println!(" ");

            let mut output = File::create(output_file)?;
            writeln!(output, "!Type:Invst")?;
            for qif in invest_transactions {
                qif.print_transaction(&mut output, cash_account, self.symbols.as_ref())?;
            }
        }

        Ok(())
    }
    pub fn print_cash_qif(&self, output_file: &PathBuf) -> Result<()> {
        let cash_only_transactions = self
            .qif_actions
            .iter()
            .filter(QifAction::cash_only)
            .collect::<Vec<_>>();
        let transaction_count = cash_only_transactions.len();
        if transaction_count != 0 {
            println!("{} cash transaction(s) found.", transaction_count);
            println!(
                "Creating .qif file for these transactions: {} .",
                output_file.as_path().display()
            );
            println!("Import this file into the appropriate bank account.");
            println!(" ");

            let mut output = File::create(output_file)?;
            writeln!(output, "!Type:Bank")?;
            for qif in cash_only_transactions {
                qif.print_transaction(&mut output, &None, self.symbols.as_ref())?;
            }
        }

        Ok(())
    }

    pub fn print_securities_qif(&self, output_file: &PathBuf) -> Result<()> {
        let symbols_opt = self.symbols.as_ref();
        match symbols_opt {
            None => Ok(()),
            Some(symbols) => {
                let mut securities = symbols.get_new_securities()?;
                securities.sort();

                let new_security_count = securities.len();

                if new_security_count == 0 {
                    println!(
                        "No new securities found.   No .qif file containing new securities generated."
                    );
                } else {
                    println!(
                        "{} new securities found with the following symbols:",
                        new_security_count
                    );
                    for security in securities.iter().by_ref() {
                        println!("\"{}\"", security.0);
                    }
                    println!(
                        "Creating .qif file for new securities : {} .   ",
                        output_file.as_path().display()
                    );
                    println!(
                        "Before importing transactions to quicken, import this securities .qif file .  "
                    );
                    println!(
                        "To avoid possible interference with existing transactions, specify a "
                    );
                    println!(
                        " non-investment account such as a bank account when importing this file."
                    );

                    let mut output = File::create(output_file)?;
                    for security in securities {
                        writeln!(output, "!Type:Security")?;

                        writeln!(output, "N{}", security.1 .0)?;
                        writeln!(output, "S{}", security.0)?;

                        match security.1 .1 {
                            SecurityType::Stock => {
                                writeln!(output, "TStock")?;
                            }

                            SecurityType::Option => {
                                writeln!(output, "TOption")?;
                            }

                            SecurityType::MutualFund => {
                                writeln!(output, "TMutual Fund")?;
                            }
                        }
                        writeln!(output, "^")?;
                    }
                }
                println!();
                Ok(())
            }
        }
    }

    pub fn print_qifs(&self, file_names: &FileNames, cash_acct: &Option<String>) -> Result<()> {
        self.print_securities_qif(&file_names.securities_qif)
            .with_context(|| {
                format!(
                    "unable to generate securities .qif file : {:#?}",
                    &file_names.securities_qif
                )
            })?;

        self.print_transactions_qif(&file_names.transactions_qif, cash_acct)
            .with_context(|| {
                format!(
                    "unable to generate investment transactions .qif file : {:#?}",
                    &file_names.transactions_qif
                )
            })?;

        if cash_acct.is_some() {
            self.print_cash_qif(&file_names.cash_qif)
                .with_context(|| {
                    format!(
                        "unable to generate cash transactions .qif file : {:#?}",
                        &file_names.cash_qif
                    )
                })?;
        }

        Ok(())
    }
}
