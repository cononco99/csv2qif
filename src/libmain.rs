use std::ffi::OsString;
use structopt::StructOpt;

use crate::file_names::FileNames;
use crate::file_to_memory;
use crate::find_matching_line::find_matching_line;
use crate::opt::Opt;
use crate::schwab_transaction::SchwabTransactions;
use crate::schwab_transaction::CsvReading;
use stable_eyre::eyre::*;
use std::collections::HashMap;

pub fn libmain<I>(iter: I) -> Result<()>
where
    I: IntoIterator,
    I::Item: Into<OsString> + Clone,
{
    let opts = Opt::from_iter(iter);
    let file_names = FileNames::new(&opts)?;

    let mut readers : HashMap<String, &dyn CsvReading> = HashMap::new();

    let schwab_reader = SchwabTransactions {};
    readers.insert(
        schwab_reader.csv_header(),
        &schwab_reader,
    );

    let mut bufreader = file_to_memory::read_file_to_cursor(&opts.transactions)?;

    let optional_reader = find_matching_line(&mut bufreader, &readers)?;

    let reader = optional_reader.ok_or(eyre!(
        "No recognized csv header found in file : {:#?}",
        &opts.transactions
    ))?;

    let transactions =
        reader
            .to_transactions(&mut bufreader, &opts.current_securities)
            .with_context(|| {
                format!(
                    "unable to read transactions .CSV file : {:#?}",
                    &opts.transactions
                )
            })?;

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
