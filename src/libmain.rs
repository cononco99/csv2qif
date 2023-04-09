use std::ffi::OsString;
use structopt::StructOpt;

use crate::file_names::FileNames;
use crate::opt::Opt;
use crate::schwab_transaction::SchwabTransaction;
use stable_eyre::eyre::*;

pub fn libmain<I>(iter: I) -> Result<()>
where
    I: IntoIterator,
    I::Item: Into<OsString> + Clone,
{
    let opts = Opt::from_iter(iter);
    let file_names = FileNames::new(&opts)?;

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
