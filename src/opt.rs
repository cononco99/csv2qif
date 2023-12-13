// Thanks to stackoverflow.com / Zeppi : https://stackoverflow.com/a/69719942/509928

use std::path::PathBuf;
use structopt::clap::arg_enum;
use structopt::StructOpt;

// Thanks to stackoverflow.com / https://stackoverflow.com/a/58171404/509928
arg_enum! {
    #[derive(Debug)]
    enum AccountType {
        Cash,
        Invest,
    }
}

#[derive(StructOpt)]
pub struct Opt {
    #[structopt(short = "a", long = "account-type", possible_values = &AccountType::variants(), case_insensitive = true)]
    account_type: AccountType,
    #[structopt(short = "w", parse(from_os_str))]
    pub workdir: Option<PathBuf>,
    #[structopt(short = "l", long = "linked")]
    pub cash_acct: Option<String>,
    #[structopt(short = "c", parse(from_os_str))]
    pub current_securities: Option<PathBuf>,
    #[structopt(parse(from_os_str))]
    pub transactions: PathBuf,
}
