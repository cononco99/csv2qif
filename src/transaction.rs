use stable_eyre::eyre::*;
use chrono::NaiveDate;


pub trait Transaction {
    fn get_date(&self) -> Result<NaiveDate>;
}
