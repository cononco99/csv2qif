use crate::symbols::Symbols;
use crate::transactions_qif::*;
use stable_eyre::eyre::*;

pub trait Transaction {
    fn to_qif_action(&self, securities: &mut Option<Symbols>) -> Result<Vec<QifAction>>;
}
