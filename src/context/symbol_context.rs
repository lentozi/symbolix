use crate::semantic::variable::Variable;
use std::collections::HashMap;

pub type Scope = HashMap<String, Variable>;

pub struct SymbolContext {
    scopes: Vec<Scope>,
}

impl SymbolContext {
    pub fn new() -> Self {
        SymbolContext { scopes: Vec::new() }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn insert(&mut self, variable: Variable) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(variable.name.clone(), variable);
        }
    }

    pub fn find(&self, name: &str) -> Option<Variable> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Some(var.clone());
            }
        }
        None
    }

    pub fn find_mut(&mut self, name: &str) -> Option<&mut Variable> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var) = scope.get_mut(name) {
                return Some(var);
            }
        }
        None
    }
}
