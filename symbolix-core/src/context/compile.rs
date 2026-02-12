use std::{
    cell::RefCell,
    sync::{Arc, RwLock},
};

use crate::{
    context::symbol_table::SymbolTable,
    semantic::variable::{Variable, VariableType},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprType {
    Numeric,
    Logical,
    Unknown,
}

pub struct CompileContext {
    pub symbol_table: RwLock<SymbolTable>,
}

impl CompileContext {
    pub fn new() -> Self {
        Self {
            symbol_table: RwLock::new(SymbolTable::new()),
        }
    }

    pub fn push_current<R>(ctx: &Arc<Self>, f: impl FnOnce(&Self) -> R) -> R {
        COMPILE_CONTEXT_STACK.with(|stack| {
            stack.borrow_mut().push(Arc::clone(ctx));
            let result = f(&*ctx);
            stack.borrow_mut().pop();
            result
        })
    }

    pub fn current() -> Option<Arc<CompileContext>> {
        COMPILE_CONTEXT_STACK.with(|stack| stack.borrow().last().cloned())
    }

    pub fn current_mut() -> Option<Arc<CompileContext>> {
        COMPILE_CONTEXT_STACK.with(|stack| stack.borrow_mut().last().cloned())
    }

    pub fn exit(&mut self) {
        COMPILE_CONTEXT_STACK.with(|stack| stack.borrow_mut().pop());
    }

    pub fn with_new_scope<R>(&self, f: impl FnOnce(&Self) -> R) -> R {
        self.enter_new_var_scope();
        let result = f(self);
        self.exit_var_scope();
        result
    }

    fn enter_new_var_scope(&self) {
        self.symbol_table
            .write()
            .expect("rwlock poisoned")
            .enter_scope();
    }

    fn exit_var_scope(&self) {
        self.symbol_table
            .write()
            .expect("rwlock poisoned")
            .exit_scope();
    }

    pub fn register_variable(&self, variable: Variable) {
        let mut table = self.symbol_table.write().expect("rwlock poisoned");
        if let Some(existing) = table.get(&variable.name) {
            if existing.var_type != variable.var_type
                && existing.var_type != VariableType::Unknown
                && variable.var_type != VariableType::Unknown
            {
                panic!("variable {} has conflicting types", variable.name);
            }
        } else {
            table.insert(variable.name.to_string(), variable);
        }
    }

    pub fn collect_variables(&self) -> Vec<Variable> {
        let table = self.symbol_table.read().expect("rwlock poisoned");
        table.collect()
    }

    pub fn collect_all_variables(&self) -> Vec<Variable> {
        let table = self.symbol_table.read().expect("rwlock poisoned");
        table.collect_all()
    }
}

thread_local! {
    static COMPILE_CONTEXT_STACK: RefCell<Vec<Arc<CompileContext>>> = RefCell::new(Vec::new());
}
