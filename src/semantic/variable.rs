use std::fmt;
use log::warn;
use crate::lexer::constant::Constant;
use crate::with_context;

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
        with_context!(ctx, {
            let symbols = &mut ctx.symbols.borrow_mut();
            match symbols.find(name) {
                Some(res) => {
                    warn!("variable '{}' already exists in the current context", name);
                    res
                }
                None => {
                    // Variable does not exist in the current context, proceed to create a new one
                    let variable = Variable {
                        name: name.to_string(),
                        var_type: var_type.clone(),
                        value: None,
                    };
                    symbols.insert(variable.clone());
                    variable
                }
            }
        })
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}