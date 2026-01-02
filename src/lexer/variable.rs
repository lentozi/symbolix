use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
}

impl Variable {
    pub fn new(name: &str) -> Variable {
        Variable {
            name: name.to_string(),
        }
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}