use std::collections::HashMap;

use crate::semantic::variable::Variable;

pub type Scope = HashMap<String, Variable>;

#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = SymbolTable { scopes: Vec::new() };
        table.enter_scope();
        table
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn insert(&mut self, name: String, symbol: Variable) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, symbol);
        }
    }

    pub fn get(&self, name: &str) -> Option<&Variable> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Variable> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(symbol) = scope.get_mut(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn collect(&self) -> Vec<Variable> {
        if let Some(current_scope) = self.scopes.last() {
            current_scope.values().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn collect_all(&self) -> Vec<Variable> {
        let mut variables = Vec::new();
        for scope in self.scopes.iter().rev() {
            for (_, symbol) in scope {
                variables.push(symbol.clone());
            }
        }
        variables
    }
}
