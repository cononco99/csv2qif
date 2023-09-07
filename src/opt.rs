// Thanks to stackoverflow.com / Zeppi : https://stackoverflow.com/a/69719942/509928

use std::path::PathBuf;

#[derive(StructOpt)]
pub struct Opt {
    #[structopt(short = "w", parse(from_os_str))]
    pub workdir: Option<PathBuf>,
    #[structopt(short = "l")]
    pub linked_acct: Option<String>,
    #[structopt(short = "c", parse(from_os_str))]
    pub current_securities: Option<PathBuf>,
    #[structopt(parse(from_os_str))]
    pub transactions: PathBuf,
}
