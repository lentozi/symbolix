use std::fmt;
use log::warn;
use crate::lexer::constant::Constant;
use crate::semantic::SYMBOL_TABLE;

#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    Integer,
    Float,
    Fraction,
    Boolean,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
    pub var_type: VariableType,
    pub value: Option<Constant>,
}

impl Variable {
    pub fn new(name: &str, var_type: VariableType) -> Variable {
        let mut guard = SYMBOL_TABLE.lock().unwrap();
        if (*guard).contains_key(name) {
            warn!("variable '{}' already exists", name);
            return (*guard).get(name).unwrap().clone();
        }
        let variable = Variable {
            name: name.to_string(),
            var_type,
            value: None,
        };
        (*guard).insert(name.to_string(), variable.clone());
        variable
    }

    pub fn find(name: &str) -> Option<Variable> {
        let guard = SYMBOL_TABLE.lock().unwrap();
        (*guard).get(name).cloned()
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}