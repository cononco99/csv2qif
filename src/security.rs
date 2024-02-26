#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityType {
    Option,
    Stock,
    MutualFund,
    MarketIndex,
}
