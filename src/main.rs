use crate::schwab_transaction::read_transactions_csv;
use crate::transactions_qif::{print_linked_qif, print_securities_qif, print_transactions_qif};
use schwab_transaction::SchwabTransaction;
use stable_eyre::eyre::*;
use std::path::PathBuf;
use structopt::StructOpt;

mod file_names;
mod schwab_transaction;
mod security;
mod symbols;
mod transactions_qif;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short = "o", parse(from_os_str))]
    outdir: Option<PathBuf>,
    #[structopt(short = "l")]
    linked_acct: Option<String>,
    #[structopt(short = "c", parse(from_os_str))]
    current_securities: PathBuf,
    #[structopt(parse(from_os_str))]
    transactions: PathBuf,
}

fn main() -> Result<()> {
    stable_eyre::install()?;
    let opts = Opt::from_args();
    let file_names = file_names::FileNames::new(&opts.outdir, &opts.transactions)?;

    let transactions_csv = read_transactions_csv(&opts.transactions).with_context(|| {
        format!(
            "unable to read transactions .CSV file : {:#?}",
            &opts.transactions
        )
    })?;

    let transactions =
        SchwabTransaction::to_transactions(&transactions_csv, &opts.current_securities)
            .with_context(|| "unable to create qif Transactions. ".to_string())?;

    print_securities_qif(&file_names.securities_qif, &transactions).with_context(|| {
        format!(
            "unable to generate securities .qif file : {:#?}",
            &file_names.securities_qif
        )
    })?;

    print_transactions_qif(
        &file_names.transactions_qif,
        &transactions,
        &opts.linked_acct,
    )
    .with_context(|| {
        format!(
            "unable to generate investment transactions .qif file : {:#?}",
            &file_names.transactions_qif
        )
    })?;

    if opts.linked_acct.is_some() {
        print_linked_qif(&file_names.linked_qif, &transactions).with_context(|| {
            format!(
                "unable to generate linked transactions .qif file : {:#?}",
                &file_names.linked_qif
            )
        })?;
    }

    Ok(())
}
