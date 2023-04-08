use regex::Regex;
use stable_eyre::eyre::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::security::SecurityType;

pub struct Symbols {
    base_symbols: HashMap<String, (String, SecurityType)>,
    new_symbols: HashMap<String, (String, SecurityType)>,
}

impl Symbols {
    pub fn new(current_securities_file: &PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(current_securities_file).with_context(|| {
            format!(
                "Unable to read from current securities file: {:#?}",
                current_securities_file
            )
        })?;

        let security_re = Regex::new(
            r"(?xm)                  
                               ^!Type:Security\r?\n
                               ^N([^\r\n]*)\r?\n
                               ^S([^\r\n]*)\r?\n
                               ^T([^\r\n]*)\r?\n
                               ",
        )
        .context("Unable to form regular expression used to match Securities")?;

        let mut base_symbols: HashMap<String, (String, SecurityType)> = HashMap::new();
        let new_symbols: HashMap<String, (String, SecurityType)> = HashMap::new();
        for security_cap in security_re.captures_iter(&contents) {
            let symbol = &security_cap[2];
            let name = &security_cap[1];
            let security_type_str = &security_cap[3];
            let security_type = match security_type_str {
                "Option" => Ok(SecurityType::Option),
                "Stock" => Ok(SecurityType::Stock),
                "Mutual Fund" => Ok(SecurityType::MutualFund),
                _ => {
                    let err_msg: String =
                        "unrecognized security type: ".to_string() + security_type_str;
                    Err(eyre!(err_msg))
                }
            }?;
            match base_symbols.entry(symbol.to_string()) {
                Entry::Occupied(o) => {
                    if o.get().0 != name {
                        println!(
                            "Symbol found multiple times in baseline securities file: {}",
                            symbol
                        );
                        println!("First name found (used by default): {}", o.get().0.clone());
                        println!("Later name found (ignored): {}", name);
                        println!();
                    }
                }
                Entry::Vacant(v) => {
                    v.insert((name.to_string(), security_type));
                }
            };
        }
        Ok(Self {
            base_symbols,
            new_symbols,
        })
    }

    pub fn lookup(&self, symbol: &String) -> Result<String> {
        let optional_base_name = self.base_symbols.get(symbol);
        match optional_base_name {
            Some((name, _)) => Ok(name.clone()),
            None => {
                let (name, _) = self.new_symbols.get(symbol).ok_or(eyre!(
                    "expected to find symbol in map: ".to_string() + symbol
                ))?;
                Ok(name.clone())
            }
        }
    }

    pub fn enter_if_not_found(
        &mut self,
        symbol: &str,
        name: &str,
        security_type: &SecurityType,
    ) -> Result<()> {
        match self.base_symbols.entry(symbol.to_owned()) {
            Entry::Occupied(_) => Ok(()),
            Entry::Vacant(_) => match self.new_symbols.entry(symbol.to_owned()) {
                Entry::Occupied(_) => Ok(()),
                Entry::Vacant(v) => {
                    v.insert((name.to_owned(), security_type.clone()));
                    Ok(())
                }
            },
        }
    }

    pub fn get_new_securities(&self) -> Result<Vec<(String, (String, SecurityType))>> {
        let mut res = Vec::new();
        for (sym, val) in self.new_symbols.iter() {
            res.push((sym.clone(), val.clone()))
        }
        Ok(res)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use crate::security::SecurityType;

    #[test]
    fn test_new() {
        // Create a test securities file
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        write!(
            temp_file,
            "!Type:Security\n\
            NTest Security 1\n\
            STEST1\n\
            TStock\n\
            !Type:Security\n\
            NTest Security 2\n\
            STEST2\n\
            TMutual Fund\n\
            "
        )
        .unwrap();

        let symbols = Symbols::new(&temp_file.path().to_path_buf()).unwrap();

        // Test that the expected symbols were read from the file
        assert_eq!(
            symbols.base_symbols.get("TEST1").unwrap(),
            &("Test Security 1".to_string(), SecurityType::Stock)
        );
        assert_eq!(
            symbols.base_symbols.get("TEST2").unwrap(),
            &("Test Security 2".to_string(), SecurityType::MutualFund)
        );
    }

    #[test]
    fn test_lookup() {
        let mut symbols = Symbols {
            base_symbols: HashMap::new(),
            new_symbols: HashMap::new(),
        };

        // Add some symbols to the map
        symbols.base_symbols.insert(
            "AAPL".to_string(),
            ("Apple Inc.".to_string(), SecurityType::Stock),
        );
        symbols.new_symbols.insert(
            "GOOG".to_string(),
            ("Alphabet Inc.".to_string(), SecurityType::Stock),
        );

        // Test that symbols can be looked up correctly
        assert_eq!(symbols.lookup(&"AAPL".to_string()).unwrap(), "Apple Inc.");
        assert_eq!(
            symbols.lookup(&"GOOG".to_string()).unwrap(),
            "Alphabet Inc."
        );

        // Test that an error is returned when looking up an unknown symbol
        assert!(symbols.lookup(&"MSFT".to_string()).is_err());
    }

    #[test]
    fn test_enter_if_not_found() {
        let mut symbols = Symbols {
            base_symbols: HashMap::new(),
            new_symbols: HashMap::new(),
        };

        // Enter a new symbol
        symbols
            .enter_if_not_found("AAPL", "Apple Inc.", &SecurityType::Stock)
            .unwrap();

        // Test that the new symbol was added
        assert_eq!(
            symbols.new_symbols.get("AAPL").unwrap(),
            &("Apple Inc.".to_string(), SecurityType::Stock)
        );

        // Try to enter the same symbol again
        symbols
            .enter_if_not_found("AAPL", "Apple Inc. (again)", &SecurityType::Stock)
            .unwrap();

        // Test that the original value was preserved and the new value was not added
        assert_eq!(
            symbols.new_symbols.get("AAPL").unwrap(),
            &("Apple Inc.".to_string(), SecurityType::Stock)
        );
    }







    #[test]
    fn test_new2() {
        let file_path = PathBuf::from("test_data/securities.txt");
        let symbols = Symbols::new(&file_path).unwrap();
        assert_eq!(symbols.base_symbols.len(), 3);
        assert_eq!(symbols.new_symbols.len(), 1);
    }

    #[test]
    fn test_lookup2() {
        let file_path = PathBuf::from("test_data/securities.txt");
        let symbols = Symbols::new(&file_path).unwrap();
        let symbol = String::from("AAPL");
        let name = symbols.lookup(&symbol).unwrap();
        assert_eq!(name, String::from("Apple Inc."));

        let symbol = String::from("XYZ");
        let name = symbols.lookup(&symbol);
        assert!(name.is_err());
    }

    #[test]
    fn test_enter_if_not_found2() {
        let file_path = PathBuf::from("test_data/securities.txt");
        let mut symbols = Symbols::new(&file_path).unwrap();

        let symbol = String::from("MSFT");
        let name = String::from("Microsoft Corporation");
        let security_type = SecurityType::Stock;
        let result = symbols.enter_if_not_found(&symbol, &name, &security_type);
        assert!(result.is_ok());

        let name = symbols.lookup(&symbol).unwrap();
        assert_eq!(name, String::from("Microsoft Corporation"));

        let symbol = String::from("XYZ");
        let name = String::from("Test Corporation");
        let security_type = SecurityType::Stock;
        let result = symbols.enter_if_not_found(&symbol, &name, &security_type);
        assert!(result.is_ok());

        let name = symbols.lookup(&symbol).unwrap();
        assert_eq!(name, String::from("Test Corporation"));
    }

    #[test]
    fn test_get_new_securities() {
        let file_path = PathBuf::from("test_data/securities.txt");
        let mut symbols = Symbols::new(&file_path).unwrap();

        let symbol = String::from("XYZ");
        let name = String::from("Test Corporation");
        let security_type = SecurityType::Stock;
        let result = symbols.enter_if_not_found(&symbol, &name, &security_type);
        assert!(result.is_ok());

        let new_securities = symbols.get_new_securities().unwrap();
        assert_eq!(new_securities.len(), 1);
        let (new_symbol, (new_name, new_security_type)) = new_securities.get(0).unwrap();
        assert_eq!(new_symbol, &symbol);
        assert_eq!(new_name, &name);
        assert_eq!(new_security_type, &security_type);
    }
}

