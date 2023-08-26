use chrono::NaiveDate;
use stable_eyre::eyre::*;

pub trait Transaction {
    fn get_date(&self) -> Result<NaiveDate>;
}
