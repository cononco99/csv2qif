#[macro_use]
extern crate structopt;

use crate::libmain::libmain;
use stable_eyre::eyre::*;

mod file_names;
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
