use chrono::{NaiveDate, Datelike};
use crate::schwab_transaction::SchwabTransaction;
use crate::error_dc::*;
use std::io::Write as IoWrite;
use std::fs::File;
use std::path::PathBuf;
use crate::symbols::*;
use crate::security::*;

#[derive (Debug)]
pub struct Transaction { 
    date: NaiveDate, 
    symbol: String,
    price: String, 
    quantity: String, 
    amount: String, 
    fees: String 
}

impl Transaction {

    fn new(schwab_transaction: &SchwabTransaction, symbols: &mut Symbols) -> Result<Self> {

        let (symbol, name, security_type) = schwab_transaction.security_details()?;

        let price = schwab_transaction.price.trim_start_matches("$").to_string();
        let quantity = schwab_transaction.quantity.clone() + if security_type == SecurityType::Option {"00"} else {""};   
        let amount = schwab_transaction.amount.trim_start_matches("$").trim_start_matches("-").to_string();
        let fees = schwab_transaction.fees.trim_start_matches("$").to_string();
        let date: NaiveDate = schwab_transaction.get_date()?;
        symbols.enter_if_not_found(&symbol, &name, &security_type)?;
        let res = Self{date, symbol, price, quantity, amount, fees};
        Ok(res)

	}

    fn expired(schwab_transaction: &SchwabTransaction, symbols: &mut Symbols) -> Result<Self> {
        let price = "".to_string();
        let amount = "".to_string();
        let fees = "".to_string();

        let (symbol, name, security_type) = schwab_transaction.security_details()?;
        if security_type != SecurityType::Option {
            return Err("Expired found in CSV for non-option".into());
        }

        // let z = if (&schwab_transaction.quantity)[0] == '-' { &schwab_transaction.quantity[1..] } else { &schwab_transaction.quantity };
        //
        let (leading, rest) = schwab_transaction.quantity.split_at(1);
        let q = if leading == "-" { rest.to_string() } else { "-".to_string() + &schwab_transaction.quantity };

        let quantity =  q + "00";
        let date: NaiveDate = schwab_transaction.get_date()?;
        symbols.enter_if_not_found(&symbol, &name, &security_type)?;
        let res = Self{date, symbol, price, quantity, amount, fees};
        Ok(res)

	}

    pub fn print(self: &Self, output: &mut dyn IoWrite, action_type: &String, linked_account: &Option<String>, symbols: &Symbols) -> Result<()> {

        let memo = symbols.lookup(&self.symbol)?;

        writeln!(output, "D{}/{}'{}", self.date.month(), self.date.day(), self.date.year() % 100)?;
        write!(output, "N{}", action_type)?;
        if let &Some(_) = linked_account {
            write!(output, "X")?;
        }
        writeln!(output, "")?;
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

#[derive (Debug)]
pub enum QifAction {
    ShtSellX         { transaction: Transaction},
    CvrShrtX         { transaction: Transaction},
    BuyX             { transaction: Transaction},
    SellX            { transaction: Transaction},
    MargIntX         { date: NaiveDate, memo: String, amount: String},
    DivX             { date: NaiveDate, symbol: String, amount: String},
    CGLongX          { date: NaiveDate, symbol: String, amount: String},
    CGShortX         { date: NaiveDate, symbol: String, amount: String},
    ShrsIn           { date: NaiveDate, symbol: String, quantity: i32},   
    LinkedAccountOnly     { date: NaiveDate, payee: String, memo: String, amount: String},   //fake
} 


impl  QifAction {

    pub fn from_schwab_transaction(schwab_transaction: &SchwabTransaction, symbols: &mut Symbols) -> Result<Vec<Self>> {

        let mut res : Vec<Self> = Vec::new();

        let csv_action = schwab_transaction.action.as_str();
        match  csv_action {
            "Sell to Open" => {
                let transaction = Transaction::new(schwab_transaction, symbols)?;
                res.push(Self::ShtSellX{transaction})
            },
            "Buy to Close" => {
                let transaction = Transaction::new(schwab_transaction, symbols)?;
                res.push(Self::CvrShrtX{transaction})
            },
            "Buy" 
            | "Buy to Open" => {
                let transaction = Transaction::new(schwab_transaction, symbols)?;
                res.push(Self::BuyX{transaction});
            },
            "Sell" 
            | "Sell to Close" => {
                let transaction = Transaction::new(schwab_transaction, symbols)?;
                res.push(Self::SellX{transaction});
            },
            "Expired" => {
                let transaction = Transaction::expired(schwab_transaction, symbols)?;
                res.push(Self::SellX{transaction});
            },
            "Margin Interest" => {
                // Margin Interest from schwab is negative but quicken wants it positive.
                // Hence the trim_start_matches hack for amount
                res.push(Self::MargIntX{ date: schwab_transaction.get_date()? 
                                 , memo:   schwab_transaction.description.clone()
                                 , amount: schwab_transaction.amount.trim_start_matches("-").to_string()});
            },
            "Cash Dividend" => {
                let (symbol, name, security_type) = schwab_transaction.security_details()?;
                symbols.enter_if_not_found(&symbol, &name, &security_type)?;
                res.push(Self::DivX{ date: schwab_transaction.get_date()?
                             , symbol
                             , amount: schwab_transaction.amount.clone()});
            },
            "Qualified Dividend" => {
                let (symbol, name, security_type) = schwab_transaction.security_details()?;
                symbols.enter_if_not_found(&symbol, &name, &security_type)?;
                res.push(Self::DivX{ date: schwab_transaction.get_date()?
                             , symbol
                             , amount: schwab_transaction.amount.clone()});
            },
            "Short Term Cap Gain" => {
                let (symbol, name, security_type) = schwab_transaction.security_details()?;
                symbols.enter_if_not_found(&symbol, &name, &security_type)?;
                res.push(Self::CGShortX{ date: schwab_transaction.get_date()?
                                 , symbol
                                 , amount: schwab_transaction.amount.clone()});
            },
            "Long Term Cap Gain" => {
                let (symbol, name, security_type) = schwab_transaction.security_details()?;
                symbols.enter_if_not_found(&symbol, &name, &security_type)?;
                res.push(Self::CGLongX{ date: schwab_transaction.get_date()?
                                , symbol
                                , amount: schwab_transaction.amount.clone()});
            },
            "Foreign Tax Paid" 
            | "ADR Mgmt Fee"
            | "Cash In Lieu" 
            | "MoneyLink Deposit" 
            | "Wire Funds" 
            | "Misc Cash Entry" 
            | "Service Fee" 
            | "Journal" 
            | "MoneyLink Transfer" 
            | "Pr Yr Cash Div" 
            | "Pr Yr Cash Div Adj" 
            | "Bank Interest" => {
                res.push(Self::LinkedAccountOnly{ date:   schwab_transaction.get_date()?
                                          , payee:  schwab_transaction.description.clone()
                                          , memo:   schwab_transaction.description.clone()
                                          , amount: schwab_transaction.amount.clone()});
            },

            "Spin-off" => {
                let (symbol, name, security_type) = schwab_transaction.security_details()?;
                let quantity = schwab_transaction.quantity.parse::<i32>()?;   
                let date: NaiveDate = schwab_transaction.get_date()?;
                symbols.enter_if_not_found(&symbol, &name, &security_type)?;
                res.push(Self::ShrsIn{date, symbol, quantity});
            },

            "Stock Split" => {
                println!("Stock Split not handled.");
                println!("This is because Schwab CSV contains the number of new shared added due to the split but quicken records the factor that the old number of shared is multiplied by to get the new number of shares.  Without knowing the starting number of shares, the factor can not be calculated.  The split will have to be entered by hand:");
                println!("{:#?}", schwab_transaction);
                println!("");
            },

            "Name Change" => {
                
                println!("Name change not handled:");
                println!("{:#?}", schwab_transaction);
                println!("");
            },

            _ => {
                if      (schwab_transaction.quantity == "") 
                     && (schwab_transaction.price == "") 
                     && (schwab_transaction.fees == "")    {

                    println!("Unrecognized action found in .CSV : \"{}\".", csv_action);

                    let linked_only = Self::LinkedAccountOnly{ date:   schwab_transaction.get_date()?
                                                     , payee:  schwab_transaction.description.clone()
                                                     , memo:   schwab_transaction.description.clone()
                                                     , amount: schwab_transaction.amount.clone()};
                    println!("No quantity, price or fees found so entering in linked account only.");
                    println!("{:#?}", linked_only);

                    res.push(linked_only);
                } else {
                    let message = "Unrecognized action: ".to_string() + schwab_transaction.action.as_str();
                    println!("{:#?}", res);
                    return Err(message.into());
                }
            }
        };
        Ok(res)

    }

    pub fn print_transaction(self: &Self, output: &mut dyn IoWrite, linked_account: &Option<String>, symbols: &Symbols) -> Result<()> {
        match self {
            Self::ShtSellX{transaction} => {
                transaction.print(output, &"ShtSell".to_string(), linked_account, symbols)
            }
            Self::CvrShrtX{transaction} => {
                transaction.print(output, &"CvrShrt".to_string(), linked_account, symbols)
            }
            Self::BuyX{transaction} => {
                transaction.print(output, &"Buy".to_string(), linked_account, symbols)
            }
            Self::SellX{transaction} => {
                transaction.print(output, &"Sell".to_string(), linked_account, symbols)
            }
            Self::MargIntX{date, memo, amount} => {
                writeln!(output, "D{}/{}'{}", date.month(), date.day(), date.year() % 100)?;
                write!(output, "NMargInt")?;
                if let &Some(_) = linked_account {
                    write!(output, "X")?;
                }
                writeln!(output, "")?;
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
            Self::LinkedAccountOnly{date, payee, memo, amount} => {
                writeln!(output, "D{}/{}'{}", date.month(), date.day(), date.year() % 100)?;
                writeln!(output, "U{}", amount)?;
                writeln!(output, "T{}", amount)?;
                writeln!(output, "P{}", payee)?;
                writeln!(output, "M{}", memo)?;
                writeln!(output, "^")?;
                Ok(())
            }
            Self::DivX{date, symbol, amount} => {
                let name = symbols.lookup(symbol)?;
                writeln!(output, "D{}/{}'{}", date.month(), date.day(), date.year() % 100)?;
                write!  (output, "NDiv")?;
                if let Some(_) = linked_account {
                    write!(output, "X")?;
                }
                writeln!(output, "")?;
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
            Self::CGLongX{date, symbol, amount} => {
                let name = symbols.lookup(symbol)?;
                writeln!(output, "D{}/{}'{}", date.month(), date.day(), date.year() % 100)?;
                write!  (output, "NCGLong")?;
                if let Some(_) = linked_account {
                    write!(output, "X")?;
                }
                writeln!(output, "")?;
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
            Self::CGShortX{date, symbol, amount} => {
                let name = symbols.lookup(symbol)?;
                writeln!(output, "D{}/{}'{}", date.month(), date.day(), date.year() % 100)?;
                write!(output, "NCGShort")?;
                if let Some(_) = linked_account {
                    write!(output, "X")?;
                }
                writeln!(output, "")?;
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
            Self::ShrsIn{date, symbol, quantity} => {
                let name = symbols.lookup(symbol)?;
                writeln!(output, "D{}/{}'{}", date.month(), date.day(), date.year() % 100)?;
                writeln!(output, "NShrsIn")?;
                writeln!(output, "Y{}", name)?;
                writeln!(output, "Q{}", quantity)?;
                writeln!(output, "M{}", name)?;
                writeln!(output, "^")?;
                Ok(())
            }
        }
    }

    fn linked_only(qa: &&Self) -> bool
    {
        match qa {
            Self::LinkedAccountOnly{date: _, payee: _, memo: _, amount: _} => {
                true
            },

            _ => { 
                false 
            },
        }
    }

    fn not_linked_only(qa: &&Self) -> bool
    {
        !Self::linked_only(qa)
    }

}

pub struct Transactions {
    qif_actions: Vec<QifAction>,
    symbols: Symbols,
}

impl Transactions {
    pub fn new(schwab_transactions: &Vec<SchwabTransaction>, current_securities_file: &PathBuf) -> Result<Transactions> {
        let schwab_transactions_reversed : Vec<SchwabTransaction> = schwab_transactions.iter().rev().cloned().collect();  // we want oldest first
        let mut symbols = Symbols::new(current_securities_file)?;

        let from_schwab_transaction = |tr| QifAction::from_schwab_transaction(tr, &mut symbols);
        let nested_actions = schwab_transactions_reversed.iter().map(from_schwab_transaction).collect::<Result<Vec<_>>>()?;
        let qif_actions = nested_actions.into_iter().flatten().collect();
        Ok(Transactions{qif_actions, symbols})
    }
}


pub fn print_transactions_qif(output_file: &PathBuf, transactions: &Transactions, linked_account: &Option<String> ) -> Result<()>
{
    let invest_transactions = if let None = &linked_account {
        transactions.qif_actions.iter().collect::<Vec<_>>()
    } else {
        transactions.qif_actions.iter().filter(QifAction::not_linked_only).collect::<Vec<_>>()
    };

    let transaction_count = invest_transactions.len();

    if transaction_count == 0 {
    } else {
        println! ("{} transaction(s) found.", transaction_count);
        // let output_file_str = output_file_str_result.map_err(|e| Err("bad file name"));
        println!("Creating .qif file for these transactions: {} .", output_file.as_path().display());
        println!("Import this file into the investment account");
        println!(" ");

        let mut output = File::create(output_file)?;
        writeln!(output, "!Type:Invst")?;
        for qif in invest_transactions {
            qif.print_transaction(&mut output, &linked_account, &transactions.symbols)?;
        }

    }

    Ok(())
}

pub fn print_linked_qif(output_file: &PathBuf, transactions: &Transactions) -> Result<()>
{
    let linked_only_transactions = transactions.qif_actions.iter().filter(QifAction::linked_only).collect::<Vec<_>>();
    let transaction_count = linked_only_transactions.len();
    if transaction_count != 0 {
        println! ("{} transaction(s) specific to linked bank account found.", transaction_count);
        println!("Creating .qif file for these transactions: {} .", output_file.as_path().display());
        println!("Import this file into the linked bank account associate with the investment account");
        println!(" ");

        let mut output = File::create(output_file)?;
        writeln!(output, "!Type:Bank")?;
        for qif in linked_only_transactions {
            qif.print_transaction(&mut output, &None, &transactions.symbols)?;
        }
    }

    Ok(())
}

pub fn print_securities_qif(output_file: &PathBuf, transactions: &Transactions) -> Result<()>
{
    let mut securities = transactions.symbols.get_new_securities()?;
    securities.sort();

    let new_security_count = securities.len();

    if new_security_count == 0 {
        println! ("No new securities found.   No .qif file containing new securities generated.");
    } else {
        println!("{} new securities found with the following symbols:", new_security_count);
        for security in securities.iter().by_ref() {
            println!("\"{}\"", security.0);
        }
        println!("Creating .qif file for new securities : {} .   ", output_file.as_path().display());
        println!("Before importing transactions to quicken, import this securities .qif file .  ");
        println!("To avoid possible interference with existing transactions, specify a ");
        println!(" non-investment account such as a bank account when importing this file.");

        let mut output = File::create(output_file)?;
        for security in securities {
            writeln!(output, "!Type:Security")?;

            writeln!(output, "N{}", security.1.0)?;
            writeln!(output, "S{}", security.0)?;

            match security.1.1{
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
    println!("");
    Ok(())
}
