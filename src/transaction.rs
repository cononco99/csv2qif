use crate::transactions_qif::*;
use chrono::NaiveDate;
use stable_eyre::eyre::*;

pub trait Transaction {
    fn get_date(&self) -> Result<NaiveDate>;

    fn to_qif_action(&self) -> Result<Vec<QifAction>>;
}
