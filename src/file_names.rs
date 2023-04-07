use stable_eyre::eyre::*;
use std::ffi::OsString;
use std::path::PathBuf;

pub struct FileNames {
    pub transactions_qif: PathBuf,
    pub linked_qif: PathBuf,
    pub securities_qif: PathBuf,
}

impl FileNames {
    // given optional output directory and required transactions file, generate output file names.
    pub fn new(outdir: &Option<PathBuf>, transactions: &PathBuf) -> Result<FileNames> {
        let outdir = outdir.clone().unwrap_or(PathBuf::from("."));
        let transactions_file_name = transactions
            .file_name()
            .with_context(|| format!("Unable to get filename from : {:#?}", &transactions))?;

        let qif_transactions_base = PathBuf::from(transactions_file_name).with_extension("qif");

        let mut transactions_qif = OsString::from("investment_transactions_");
        transactions_qif.push(&qif_transactions_base);
        let transactions_qif_pathbuf = outdir.join(PathBuf::from(&transactions_qif));

        let mut linked_qif = OsString::from("linked_transactions_");
        linked_qif.push(&qif_transactions_base);
        let linked_qif_pathbuf = outdir.join(PathBuf::from(&linked_qif));

        let mut securities_qif = OsString::from("securities_");
        securities_qif.push(&qif_transactions_base);
        let securities_qif_pathbuf = outdir.join(PathBuf::from(&securities_qif));

        let filenames = FileNames {
            transactions_qif: transactions_qif_pathbuf,
            linked_qif: linked_qif_pathbuf,
            securities_qif: securities_qif_pathbuf,
        };
        Ok(filenames)
    }
}
