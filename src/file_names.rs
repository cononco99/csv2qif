use crate::opt::Opt;
use stable_eyre::eyre::*;
use std::ffi::OsString;
use std::path::PathBuf;

pub struct FileNames {
    pub transactions_qif: PathBuf,
    pub linked_qif: PathBuf,
    pub securities_qif: PathBuf,
    pub workdir: PathBuf,
}

impl FileNames {
    // given optional output directory and required transactions file, generate output file names.
    pub fn new(opts: &Opt) -> Result<FileNames> {
        let transactions_file_name = opts
            .transactions
            .file_name()
            .with_context(|| format!("Unable to get filename from : {:#?}", &opts.transactions))?;

        let qif_transactions_base = PathBuf::from(transactions_file_name).with_extension("qif");

        let mut t = OsString::from("investment_");
        t.push(&qif_transactions_base);
        let transactions_qif = PathBuf::from(&t);

        let mut t = OsString::from("");
        t.push(&qif_transactions_base);
        let linked_qif = PathBuf::from(&t);

        let mut t = OsString::from("securities_");
        t.push(&qif_transactions_base);
        let securities_qif = PathBuf::from(&t);

        let filenames = FileNames {
            transactions_qif,
            linked_qif,
            securities_qif,
            workdir: opts.workdir.clone().unwrap_or(PathBuf::from(".")),
        };
        Ok(filenames)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() -> Result<(), Box<dyn std::error::Error>> {
        /*
                // Test with no output directory specified
                let transactions = PathBuf::from("sample.csv");
                let filenames = FileNames::new(&None, &transactions)?;
                assert_eq!(
                    filenames.transactions_qif,
                    PathBuf::from("./investment_transactions_sample.qif")
                );
                assert_eq!(
                    filenames.linked_qif,
                    PathBuf::from("./linked_transactions_sample.qif")
                );
                assert_eq!(
                    filenames.securities_qif,
                    PathBuf::from("./securities_sample.qif")
                );

                // Test with output directory specified
                let outdir = PathBuf::from("output");
                let filenames = FileNames::new(&Some(outdir.clone()), &transactions)?;
                assert_eq!(
                    filenames.transactions_qif,
                    outdir.join(PathBuf::from("investment_transactions_sample.qif"))
                );
                assert_eq!(
                    filenames.linked_qif,
                    outdir.join(PathBuf::from("linked_transactions_sample.qif"))
                );
                assert_eq!(
                    filenames.securities_qif,
                    outdir.join(PathBuf::from("securities_sample.qif"))
                );
        */

        Ok(())
    }
}
