use std::ffi::OsString;
use structopt::StructOpt;

use crate::file_names::FileNames;
use crate::file_to_memory;
use crate::find_matching_line::find_matching_line;
use crate::opt::Opt;
use crate::schwab_transaction::{SchwabTransactions, SchwabTransaction};
use stable_eyre::eyre::*;
use std::collections::HashMap;



pub fn libmain<I>(iter: I) -> Result<()>
where
    I: IntoIterator,
    I::Item: Into<OsString> + Clone,
{
    let opts = Opt::from_iter(iter);
    let file_names = FileNames::new(&opts)?;

    let mut readers = HashMap::new();
    readers.insert("xxx".to_string(), SchwabTransactions::new()?);

    let mut bufreader = file_to_memory::read_file_to_cursor(&opts.transactions)?;

    let optional_reader = find_matching_line(&mut bufreader, &readers)?;

    let  mut reader = optional_reader.ok_or(eyre!(
        "No recognized csv header found in file".to_string() )
    )?;

    let transactions_csv = reader.read_transactions_csv(&mut bufreader)
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
