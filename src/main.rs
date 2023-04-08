#[macro_use] extern crate structopt;
use structopt::StructOpt;

use stable_eyre::eyre::*;
use schwab_transaction::SchwabTransaction;

mod opt;
mod file_names;
mod schwab_transaction;
mod security;
mod symbols;
mod transactions_qif;

fn main() -> Result<()> {
    stable_eyre::install()?;
    let opts = opt::Opt::from_args();
    let file_names = file_names::FileNames::new(&opts)?;

    let transactions_csv = SchwabTransaction::read_transactions_csv(&opts.transactions)
        .with_context(|| {
            format!(
                "unable to read transactions .CSV file : {:#?}",
                &opts.transactions
            )
        })?;

    let transactions =
        SchwabTransaction::to_transactions(&transactions_csv, &opts.current_securities)
            .with_context(|| "unable to create qif Transactions. ".to_string())?;

    transactions
        .print_securities_qif(&file_names.securities_qif)
        .with_context(|| {
            format!(
                "unable to generate securities .qif file : {:#?}",
                &file_names.securities_qif
            )
        })?;

    transactions
        .print_transactions_qif(&file_names.transactions_qif, &opts.linked_acct)
        .with_context(|| {
            format!(
                "unable to generate investment transactions .qif file : {:#?}",
                &file_names.transactions_qif
            )
        })?;

    if let Some(linked_qif) = file_names.linked_qif {
        transactions
            .print_linked_qif(&linked_qif)
            .with_context(|| {
                format!(
                    "unable to generate linked transactions .qif file : {:#?}",
                    &linked_qif
                )
            })?;
    }

    Ok(())
}
