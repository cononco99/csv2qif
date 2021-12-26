use structopt::StructOpt;
use std::ffi::OsString;
use std::path::PathBuf;
use anyhow::*;
use crate::schwab_transaction::read_transactions_csv;
use crate::transactions_qif::{Transactions, print_transactions_qif, print_linked_qif, print_securities_qif};

mod schwab_transaction;
mod transactions_qif;
mod symbols;
mod security;


#[derive(StructOpt)]
struct Opt {
    #[structopt(short = "l")]
    linked_acct: Option<String>,
    #[structopt(short = "c",parse(from_os_str))]
    current_securities: PathBuf,
    #[structopt(parse(from_os_str))]
    transactions: PathBuf,

}

fn main() -> Result<()> {
    let opts = Opt::from_args();
    let mut qif_transactions_base : PathBuf = PathBuf::from(&opts.transactions.file_name().ok_or(anyhow!("Unable to get filename"))?);
    qif_transactions_base.set_extension("qif");

    let mut transactions_qif_filename = OsString::from("investment_transactions_");
    transactions_qif_filename.push(&qif_transactions_base);

    let mut linked_qif_filename = OsString::from("linked_transactions_");
    linked_qif_filename.push(&qif_transactions_base);

    let mut securities_qif_filename = OsString::from("securities_");
    securities_qif_filename.push(&qif_transactions_base);


    let transactions_csv = read_transactions_csv(&opts.transactions)?;
    let transactions = Transactions::new(&transactions_csv, &opts.current_securities)?;
    print_securities_qif(&PathBuf::from(securities_qif_filename), &transactions)?;
    print_transactions_qif(&PathBuf::from(&transactions_qif_filename), &transactions, &opts.linked_acct)?;
    if let Some(_) = &opts.linked_acct {
        print_linked_qif(&PathBuf::from(&linked_qif_filename), &transactions)?;
    }
    Ok(())

}
