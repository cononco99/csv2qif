extern crate structopt;

use crate::libmain::libmain;
use stable_eyre::eyre::*;

mod csv_reader;
// mod fidelity_reader;
mod file_names;
mod file_to_memory;
mod find_matching_line;
mod libmain;
mod opt;
mod readers;
mod schwab_reader;
mod security;
mod sofi_reader;
mod symbols;
mod transaction;
mod transactions_qif;

fn main() -> Result<()> {
    stable_eyre::install()?;
    let args = std::env::args();
    libmain(args)
}
