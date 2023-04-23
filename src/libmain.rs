use std::ffi::OsString;
use structopt::StructOpt;

use crate::file_names::FileNames;
use crate::file_to_memory;
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

    let mut bufreader = file_to_memory::read_file_to_cursor(&opts.transactions)?;

    let transactions_csv = SchwabTransaction::read_transactions_csv(&mut bufreader)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_libmain() -> Result<()> {
        // fix later
        let _args = "programname -c sdfsdf -l xxxxx sdsdfsdf.txt";
        // libmain(args.split(" "))
        Ok(())
    }
}
