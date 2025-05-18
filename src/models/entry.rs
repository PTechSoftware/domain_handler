use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub token: String,
    pub activated: bool,
    pub txt: Option<String>,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let txt = match &self.txt {
            Some(v) => v,
            None => &String::new()            
        };
        writeln!(f, "Name: {}, Activated:{}, txt: {} ", self.name, self.activated,txt )
    }
}