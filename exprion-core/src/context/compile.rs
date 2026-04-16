use std::{
    cell::RefCell,
    sync::{Arc, RwLock},
};

use crate::{
    context::symbol_table::SymbolTable,
    error::ErrorExt,
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
    pub error_queue: RwLock<Vec<ErrorExt>>,
}

impl CompileContext {
    pub fn new() -> Self {
        Self {
            symbol_table: RwLock::new(SymbolTable::new()),
            error_queue: RwLock::new(Vec::new()),
        }
    }

    pub fn push_current<R>(ctx: &Arc<Self>, f: impl FnOnce(&Self) -> R) -> R {
        COMPILE_CONTEXT_STACK.with(|stack| {
            stack.borrow_mut().push(Arc::clone(ctx));
            let result = f(&*ctx);
            let ctx = stack.borrow_mut().pop().unwrap();
            ctx.print_errors();
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

    pub fn search_variable(&self, variable_name: &str) -> Option<Variable> {
        let table = self.symbol_table.write().expect("rwlock poisoned");
        if let Some(existing) = table.get(variable_name) {
            Some(existing.clone())
        } else {
            None
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

    pub fn push_error(&self, error: ErrorExt) {
        if error.is_fatal() {
            self.print_errors();
            panic!("fatal error: {}", error.error_message());
        } else {
            let mut errors = self.error_queue.write().expect("rwlock poisoned");
            errors.push(error.clone());
        }
    }

    pub fn print_errors(&self) {
        let errors = self.error_queue.read().expect("rwlock poisoned");
        for error in errors.iter() {
            println!("Error {}: {}", error.error_id(), error.error_message());
        }
    }
}

thread_local! {
    static COMPILE_CONTEXT_STACK: RefCell<Vec<Arc<CompileContext>>> = RefCell::new(Vec::new());
}
