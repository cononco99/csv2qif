#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[allow(dead_code)]
pub fn general_error<T>(s:&str) -> Result<T> {
    Err(Box::new(Error::General(String::from(s))))
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    General(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::General(message) => write!(f, "General({})", message),
        }
    }
}

impl std::error::Error for Error {}
