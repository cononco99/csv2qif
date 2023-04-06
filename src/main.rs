use structopt::StructOpt;
use std::ffi::OsString;
use std::path::PathBuf;
use stable_eyre::eyre::*;
use crate::schwab_transaction::read_transactions_csv;
use crate::transactions_qif::{Transactions, print_transactions_qif, print_linked_qif, print_securities_qif};

mod schwab_transaction;
mod transactions_qif;
mod symbols;
mod security;


#[derive(StructOpt)]
struct Opt {
    #[structopt(short = "o", parse(from_os_str))]
    outdir: Option<PathBuf>,
    #[structopt(short = "l")]
    linked_acct: Option<String>,
    #[structopt(short = "c",parse(from_os_str))]
    current_securities: PathBuf,
    #[structopt(parse(from_os_str))]
    transactions: PathBuf,

}

fn main() -> Result<()> {
    stable_eyre::install()?;
    let opts = Opt::from_args();
    let outdir = opts.outdir.clone().unwrap_or(PathBuf::from("."));
    let transactions_file_name = opts.transactions.file_name()
          .with_context(|| format!("Unable to get filename from : {:#?}", &opts.transactions))?;

    let qif_transactions_base = PathBuf::from(transactions_file_name).with_extension("qif");

    let mut transactions_qif_filename = OsString::from("investment_transactions_");
    transactions_qif_filename.push(&qif_transactions_base);
    let transactions_qif_pathbuf = outdir.clone().join(PathBuf::from(&transactions_qif_filename));

    let mut linked_qif_filename = OsString::from("linked_transactions_");
    linked_qif_filename.push(&qif_transactions_base);
    let linked_qif_pathbuf = outdir.clone().join(PathBuf::from(&linked_qif_filename));

    let mut securities_qif_filename = OsString::from("securities_");
    securities_qif_filename.push(&qif_transactions_base);
    let securities_qif_pathbuf = outdir.clone().join(PathBuf::from(&securities_qif_filename));

    let transactions_csv = read_transactions_csv(&opts.transactions)
          .with_context(|| format!("unable to read transactions .CSV file : {:#?}", &opts.transactions))?;

    let transactions = Transactions::new(&transactions_csv, &opts.current_securities)
          .with_context(|| format!("unable to create qif Transactions. "))?;

    print_securities_qif(&securities_qif_pathbuf, &transactions)
          .with_context(|| format!("unable to generate securities .qif file : {:#?}", &securities_qif_filename))?;

    print_transactions_qif(&transactions_qif_pathbuf, &transactions, &opts.linked_acct)
          .with_context(|| format!("unable to generate investment transactions .qif file : {:#?}", &transactions_qif_filename))?;

    if let Some(_) = &opts.linked_acct {
        print_linked_qif(&linked_qif_pathbuf, &transactions)
            .with_context(|| format!("unable to generate linked transactions .qif file : {:#?}", &linked_qif_filename))?;
    }

    Ok(())

}
