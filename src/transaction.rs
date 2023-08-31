use crate::transactions_qif::*;
use crate::symbols::Symbols;
use chrono::NaiveDate;
use stable_eyre::eyre::*;

pub trait Transaction {
    fn get_date(&self) -> Result<NaiveDate>;

    fn to_qif_action(&self, securities: &mut Option<Symbols>) -> Result<Vec<QifAction>>;
}
