use std::collections::HashMap;

use crate::semantic::variable::{Variable, VariableType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprType {
    Numeric,
    Logical,
    Unknown,
}

pub struct CompileContext {
    pub variables: HashMap<String, Variable>,
}

impl CompileContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn register_variable(&mut self, variable: Variable) {
        if let Some(existing) = self.variables.get(&variable.name) {
            if existing.var_type != variable.var_type
                && existing.var_type != VariableType::Unknown
                && variable.var_type != VariableType::Unknown
            {
                panic!("variable {} has conflicting types", variable.name);
            } else {
                self.variables.insert(variable.name.to_string(), variable);
            }
        }
    }
}
