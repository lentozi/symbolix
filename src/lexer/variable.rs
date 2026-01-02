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