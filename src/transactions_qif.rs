use chrono::{Datelike, NaiveDate};
use stable_eyre::eyre::*;
use std::fs::File;
use std::io::Write as IoWrite;
use std::path::PathBuf;

use crate::security::*;
use crate::symbols::*;

#[derive(Debug)]
pub struct Transaction {
    pub date: NaiveDate,
    pub symbol: String,
    pub price: String,
    pub quantity: String,
    pub amount: String,
    pub fees: String,
}

impl Transaction {
    pub fn print(
        &self,
        output: &mut dyn IoWrite,
        action_type: &String,
        linked_account: &Option<String>,
        symbols: &Symbols,
    ) -> Result<()> {
        let memo = symbols.lookup(&self.symbol)?;

        writeln!(
            output,
            "D{}/{}'{}",
            self.date.month(),
            self.date.day(),
            self.date.year() % 100
        )?;
        write!(output, "N{}", action_type)?;
        if linked_account.is_some() {
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
        if let Some(acctname) = linked_account {
            writeln!(output, "L[{}]", acctname)?
        }
        writeln!(output, "${}", self.amount)?;
        writeln!(output, "^")?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum QifAction {
    ShtSellX {
        transaction: Transaction,
    },
    CvrShrtX {
        transaction: Transaction,
    },
    BuyX {
        transaction: Transaction,
    },
    SellX {
        transaction: Transaction,
    },
    MargIntX {
        date: NaiveDate,
        memo: String,
        amount: String,
    },
    DivX {
        date: NaiveDate,
        symbol: String,
        amount: String,
    },
    CGLongX {
        date: NaiveDate,
        symbol: String,
        amount: String,
    },
    CGShortX {
        date: NaiveDate,
        symbol: String,
        amount: String,
    },
    ShrsIn {
        date: NaiveDate,
        symbol: String,
        quantity: i32,
    },
    LinkedAccountOnly {
        date: NaiveDate,
        payee: String,
        memo: String,
        amount: String,
    }, //fake
}

impl QifAction {
    pub fn print_transaction(
        &self,
        output: &mut dyn IoWrite,
        linked_account: &Option<String>,
        symbols: &Symbols,
    ) -> Result<()> {
        match self {
            Self::ShtSellX { transaction } => {
                transaction.print(output, &"ShtSell".to_string(), linked_account, symbols)
            }
            Self::CvrShrtX { transaction } => {
                transaction.print(output, &"CvrShrt".to_string(), linked_account, symbols)
            }
            Self::BuyX { transaction } => {
                transaction.print(output, &"Buy".to_string(), linked_account, symbols)
            }
            Self::SellX { transaction } => {
                transaction.print(output, &"Sell".to_string(), linked_account, symbols)
            }
            Self::MargIntX { date, memo, amount } => {
                writeln!(
                    output,
                    "D{}/{}'{}",
                    date.month(),
                    date.day(),
                    date.year() % 100
                )?;
                write!(output, "NMargInt")?;
                if linked_account.is_some() {
                    write!(output, "X")?;
                }
                writeln!(output)?;
                writeln!(output, "U{}", amount)?;
                writeln!(output, "T{}", amount)?;
                writeln!(output, "M{}", memo)?;
                if let Some(acctname) = linked_account {
                    writeln!(output, "L[{}]", acctname)?
                }
                writeln!(output, "${}", amount)?;
                writeln!(output, "^")?;
                Ok(())
            }
            Self::LinkedAccountOnly {
                date,
                payee,
                memo,
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
                writeln!(output, "M{}", memo)?;
                writeln!(output, "^")?;
                Ok(())
            }
            Self::DivX {
                date,
                symbol,
                amount,
            } => {
                let name = symbols.lookup(symbol)?;
                writeln!(
                    output,
                    "D{}/{}'{}",
                    date.month(),
                    date.day(),
                    date.year() % 100
                )?;
                write!(output, "NDiv")?;
                if linked_account.is_some() {
                    write!(output, "X")?;
                }
                writeln!(output)?;
                writeln!(output, "Y{}", name)?;
                writeln!(output, "U{}", amount)?;
                writeln!(output, "T{}", amount)?;
                writeln!(output, "M{}", name)?;
                if let Some(acctname) = linked_account {
                    writeln!(output, "L[{}]", acctname)?
                }
                writeln!(output, "${}", amount)?;
                writeln!(output, "^")?;
                Ok(())
            }
            Self::CGLongX {
                date,
                symbol,
                amount,
            } => {
                let name = symbols.lookup(symbol)?;
                writeln!(
                    output,
                    "D{}/{}'{}",
                    date.month(),
                    date.day(),
                    date.year() % 100
                )?;
                write!(output, "NCGLong")?;
                if linked_account.is_some() {
                    write!(output, "X")?;
                }
                writeln!(output)?;
                writeln!(output, "Y{}", name)?;
                writeln!(output, "U{}", amount)?;
                writeln!(output, "T{}", amount)?;
                writeln!(output, "M{}", name)?;
                if let Some(acctname) = linked_account {
                    writeln!(output, "L[{}]", acctname)?
                }
                writeln!(output, "${}", amount)?;
                writeln!(output, "^")?;
                Ok(())
            }
            Self::CGShortX {
                date,
                symbol,
                amount,
            } => {
                let name = symbols.lookup(symbol)?;
                writeln!(
                    output,
                    "D{}/{}'{}",
                    date.month(),
                    date.day(),
                    date.year() % 100
                )?;
                write!(output, "NCGShort")?;
                if linked_account.is_some() {
                    write!(output, "X")?;
                }
                writeln!(output)?;
                writeln!(output, "Y{}", name)?;
                writeln!(output, "U{}", amount)?;
                writeln!(output, "T{}", amount)?;
                writeln!(output, "M{}", name)?;
                if let Some(acctname) = linked_account {
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
                let name = symbols.lookup(symbol)?;
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

    fn linked_only(qa: &&Self) -> bool {
        matches!(
            qa,
            Self::LinkedAccountOnly {
                date: _,
                payee: _,
                memo: _,
                amount: _,
            }
        )
    }

    fn not_linked_only(qa: &&Self) -> bool {
        !Self::linked_only(qa)
    }
}

pub struct Transactions {
    pub qif_actions: Vec<QifAction>,
    pub symbols: Symbols,
}

pub fn print_transactions_qif(
    output_file: &PathBuf,
    transactions: &Transactions,
    linked_account: &Option<String>,
) -> Result<()> {
    let invest_transactions = if linked_account.is_none() {
        transactions.qif_actions.iter().collect::<Vec<_>>()
    } else {
        transactions
            .qif_actions
            .iter()
            .filter(QifAction::not_linked_only)
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
            qif.print_transaction(&mut output, linked_account, &transactions.symbols)?;
        }
    }

    Ok(())
}

pub fn print_linked_qif(output_file: &PathBuf, transactions: &Transactions) -> Result<()> {
    let linked_only_transactions = transactions
        .qif_actions
        .iter()
        .filter(QifAction::linked_only)
        .collect::<Vec<_>>();
    let transaction_count = linked_only_transactions.len();
    if transaction_count != 0 {
        println!(
            "{} transaction(s) specific to linked bank account found.",
            transaction_count
        );
        println!(
            "Creating .qif file for these transactions: {} .",
            output_file.as_path().display()
        );
        println!(
            "Import this file into the linked bank account associate with the investment account"
        );
        println!(" ");

        let mut output = File::create(output_file)?;
        writeln!(output, "!Type:Bank")?;
        for qif in linked_only_transactions {
            qif.print_transaction(&mut output, &None, &transactions.symbols)?;
        }
    }

    Ok(())
}

impl Transactions {
pub fn print_securities_qif(self: &Self, output_file: &PathBuf) -> Result<()> {
    let mut securities = self.symbols.get_new_securities()?;
    securities.sort();

    let new_security_count = securities.len();

    if new_security_count == 0 {
        println!("No new securities found.   No .qif file containing new securities generated.");
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
        println!("Before importing transactions to quicken, import this securities .qif file .  ");
        println!("To avoid possible interference with existing transactions, specify a ");
        println!(" non-investment account such as a bank account when importing this file.");

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
