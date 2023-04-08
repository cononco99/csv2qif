#[macro_use]
extern crate structopt;
use structopt::StructOpt;

use schwab_transaction::SchwabTransaction;
use stable_eyre::eyre::*;

mod file_names;
mod opt;
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
        .print_qifs(&file_names, &opts.linked_acct)
        .with_context(|| "unable to create qif files. ".to_string())?;

    Ok(())
}
