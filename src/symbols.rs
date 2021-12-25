use regex::Regex;
use std::fs;
use std::path::PathBuf;
use crate::error_dc::*;
use crate::security::*;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

pub struct Symbols {
    base_symbols:  HashMap<String, (String, SecurityType)>,
    new_symbols:  HashMap<String, (String, SecurityType)>,
}

impl Symbols {
    pub fn new(current_securities_file: &PathBuf) -> Result<Self>
    {
        let contents = fs::read_to_string(&current_securities_file)?;

        // funky [\r\n]+ endings because quicken sometimes throwns in blank lines in QIF files.
        let security_re 
                = Regex::new(r"(?xm)                  
                               ^!Type:Security\r?\n
                               ^N([^\r\n]*)\r?\n
                               ^S([^\r\n]*)\r?\n
                               ^T([^\r\n]*)\r?\n
                               "
                            )?;

        let mut base_symbols : HashMap<String, (String, SecurityType)> = HashMap::new();
        let new_symbols : HashMap<String, (String, SecurityType)> = HashMap::new();
        for security_cap in security_re.captures_iter(&contents) {
            let symbol = &security_cap[2];
            let name = &security_cap[1];
            let security_type_str = &security_cap[3];
            let security_type = match security_type_str {
                "Option" => Ok(SecurityType::Option),
                "Stock" => Ok(SecurityType::Stock),
                "Mutual Fund" => Ok(SecurityType::MutualFund),
                _ => {
                    let err_msg : String =  "unrecognized security type: ".to_string() + security_type_str;
                    let boxed_err_msg : Box<dyn std::error::Error> = err_msg.into();
                    Err(boxed_err_msg)
                }
            }?;
            match base_symbols.entry(symbol.to_string()) {
                    Entry::Occupied(o) => {
                        if o.get().0 != name
                        {
                            println!("Symbol found multiple times in baseline securities file: {}", symbol.to_string());
                            println!("First name found (used by default): {}", o.get().0.clone());
                            println!("Later name found (ignored): {}", name.clone());
                            println!("");
                        }
                        ()
                    },
                    Entry::Vacant(v) => 
                    {
                        v.insert((name.to_string(), security_type));
                        ()
                    }
            };

        }
        Ok(Self{base_symbols,new_symbols})
    }

    pub fn lookup(&self, symbol: &String) -> Result<String>
    {
         let optional_base_name = self.base_symbols.get(symbol);
         match optional_base_name {
             Some((name,_)) => {
                 Ok(name.clone())
             }
             None => {
                 let (name,_) = self.new_symbols.get(symbol).ok_or("expected to find symbol in map: ".to_string() + &symbol)?;
                 Ok(name.clone())
             }
         }
    }

    pub fn enter_if_not_found(&mut self, symbol: &String, name: &String, security_type: &SecurityType) -> Result<()>
    {
        match self.base_symbols.entry(symbol.clone()) {
            Entry::Occupied(_) => {
                Ok(())
            },
            Entry::Vacant(_) => {
                match self.new_symbols.entry(symbol.clone()) {
                    Entry::Occupied(_) => {
                        Ok(())
                    },
                    Entry::Vacant(v) => 
                    {
                        v.insert((name.clone(), security_type.clone()));
                        Ok(())
                    }
                }
            }
        }
    }

    pub fn get_new_securities(&self) -> Result<Vec<(String, (String, SecurityType))>>
    {
        let mut res = Vec::new();
        for (sym, val) in self.new_symbols.iter()
        {
            res.push ((sym.clone(), val.clone()))

        }
        Ok(res)
    }
}
