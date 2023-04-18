#[macro_use]
extern crate structopt;

use crate::libmain::libmain;
use stable_eyre::eyre::*;

mod csv_key;
mod file_names;
mod file_to_memory;
mod find_matching_line;
mod libmain;
mod opt;
mod schwab_transaction;
mod security;
mod symbols;
mod transactions_qif;

fn main() -> Result<()> {
    stable_eyre::install()?;
    let args = std::env::args();
    libmain(args)
}
