use chrono::NaiveDate;
use stable_eyre::eyre::*;
use crate::transactions_qif::*;

pub trait Transaction {
    fn get_date(&self) -> Result<NaiveDate>;

    fn to_qif_action(sofi_transaction: &Self) -> Result<Vec<QifAction>> ;
}
