use std::collections::HashMap;

use crate::semantic::variable::Variable;

pub type Scope = HashMap<String, Variable>;

#[derive(Debug, Clone)]
pub struct VariableStack {
    scopes: Vec<Scope>,
}

pub type SymbolTable = VariableStack;

impl VariableStack {
    pub fn new() -> Self {
        let mut stack = VariableStack { scopes: Vec::new() };
        stack.push_scope();
        stack
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn current_scope(&self) -> Option<&Scope> {
        self.scopes.last()
    }

    pub fn current_scope_mut(&mut self) -> Option<&mut Scope> {
        self.scopes.last_mut()
    }

    pub fn define(&mut self, variable: Variable) {
        if let Some(scope) = self.current_scope_mut() {
            scope.insert(variable.name.clone(), variable);
        }
    }

    pub fn resolve(&self, name: &str) -> Option<&Variable> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
    }

    pub fn resolve_mut(&mut self, name: &str) -> Option<&mut Variable> {
        self.scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.get_mut(name))
    }

    pub fn collect_current(&self) -> Vec<Variable> {
        self.current_scope()
            .map(|scope| scope.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn collect_all(&self) -> Vec<Variable> {
        let mut variables = Vec::new();
        for scope in self.scopes.iter().rev() {
            variables.extend(scope.values().cloned());
        }
        variables
    }

    pub fn enter_scope(&mut self) {
        self.push_scope();
    }

    pub fn exit_scope(&mut self) {
        self.pop_scope();
    }

    pub fn insert(&mut self, name: String, symbol: Variable) {
        if let Some(scope) = self.current_scope_mut() {
            scope.insert(name, symbol);
        }
    }

    pub fn get(&self, name: &str) -> Option<&Variable> {
        self.resolve(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Variable> {
        self.resolve_mut(name)
    }

    pub fn collect(&self) -> Vec<Variable> {
        self.collect_current()
    }
}
