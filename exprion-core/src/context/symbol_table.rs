use std::collections::HashMap;

use crate::semantic::variable::Variable;

pub type NameId = u32;
pub type Scope = HashMap<NameId, Variable>;

#[derive(Debug, Clone)]
pub struct VariableStack {
    scopes: Vec<Scope>,
    names: HashMap<String, NameId>,
    next_name_id: NameId,
}

pub type SymbolTable = VariableStack;

impl VariableStack {
    pub fn new() -> Self {
        let mut stack = VariableStack {
            scopes: Vec::new(),
            names: HashMap::new(),
            next_name_id: 0,
        };
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
        let name_id = if variable.name_id != 0 {
            variable.name_id
        } else {
            self.intern_name(&variable.name)
        };
        let mut variable = variable;
        variable.name_id = name_id;
        if let Some(scope) = self.current_scope_mut() {
            scope.insert(name_id, variable);
        }
    }

    pub fn resolve(&self, name: &str) -> Option<&Variable> {
        let name_id = self.name_id(name)?;
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(&name_id))
    }

    pub fn resolve_mut(&mut self, name: &str) -> Option<&mut Variable> {
        let name_id = self.name_id(name)?;
        self.scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.get_mut(&name_id))
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
        let name_id = self.intern_name(&name);
        if let Some(scope) = self.current_scope_mut() {
            scope.insert(name_id, symbol);
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

    pub fn name_id(&self, name: &str) -> Option<NameId> {
        self.names.get(name).copied()
    }

    pub fn intern_name(&mut self, name: &str) -> NameId {
        if let Some(id) = self.name_id(name) {
            return id;
        }

        let id = self.next_name_id;
        self.next_name_id = self.next_name_id.wrapping_add(1);
        self.names.insert(name.to_string(), id);
        id
    }
}
