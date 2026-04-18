use std::{
    cell::RefCell,
    rc::Rc,
};

use crate::{
    context::symbol_table::SymbolTable,
    error::ErrorExt,
    lexer::constant::Constant,
    semantic::variable::{Variable, VariableType},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprType {
    Numeric,
    Logical,
    Unknown,
}

pub struct CompileContext {
    pub symbol_table: RefCell<SymbolTable>,
    pub error_queue: RefCell<Vec<ErrorExt>>,
}

impl CompileContext {
    pub fn new() -> Self {
        Self {
            symbol_table: RefCell::new(SymbolTable::new()),
            error_queue: RefCell::new(Vec::new()),
        }
    }

    pub fn push_current<R>(ctx: &Rc<Self>, f: impl FnOnce(&Self) -> R) -> R {
        COMPILE_CONTEXT_STACK.with(|stack| {
            stack.borrow_mut().push(Rc::clone(ctx));
            let result = f(&*ctx);
            let ctx = stack.borrow_mut().pop().unwrap();
            ctx.print_errors();
            result
        })
    }

    pub fn current() -> Option<Rc<CompileContext>> {
        COMPILE_CONTEXT_STACK.with(|stack| stack.borrow().last().cloned())
    }

    pub fn current_mut() -> Option<Rc<CompileContext>> {
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
        self.symbol_table.borrow_mut().push_scope();
    }

    fn exit_var_scope(&self) {
        self.symbol_table.borrow_mut().pop_scope();
    }

    pub fn register_variable(&self, variable: Variable) {
        let mut table = self.symbol_table.borrow_mut();
        if let Some(existing) = table.get(&variable.name) {
            if existing.var_type != variable.var_type
                && existing.var_type != VariableType::Unknown
                && variable.var_type != VariableType::Unknown
            {
                panic!("variable {} has conflicting types", variable.name);
            }
        } else {
            table.define(variable);
        }
    }

    pub fn search_variable(&self, variable_name: &str) -> Option<Variable> {
        self.symbol_table.borrow().get(variable_name).cloned()
    }

    pub fn resolve_variable(&self, variable_name: &str, var_type: VariableType) -> Variable {
        self.resolve_variable_with_value(variable_name, var_type, None)
    }

    pub fn resolve_variable_with_value(
        &self,
        variable_name: &str,
        var_type: VariableType,
        init_val: Option<Constant>,
    ) -> Variable {
        let mut table = self.symbol_table.borrow_mut();
        if let Some(existing) = table.get(variable_name) {
            return existing.clone();
        }

        let name_id = table.intern_name(variable_name);
        let variable = Variable {
            name_id,
            name: variable_name.to_string(),
            var_type,
            value: init_val,
        };
        table.define(variable.clone());
        variable
    }

    pub fn collect_variables(&self) -> Vec<Variable> {
        let table = self.symbol_table.borrow();
        table.collect()
    }

    pub fn collect_all_variables(&self) -> Vec<Variable> {
        let table = self.symbol_table.borrow();
        table.collect_all()
    }

    pub fn push_error(&self, error: ErrorExt) {
        if error.is_fatal() {
            self.print_errors();
            panic!("fatal error: {}", error.error_message());
        } else {
            let mut errors = self.error_queue.borrow_mut();
            errors.push(error.clone());
        }
    }

    pub fn print_errors(&self) {
        let errors = self.error_queue.borrow();
        for error in errors.iter() {
            println!("Error {}: {}", error.error_id(), error.error_message());
        }
    }
}

thread_local! {
    static COMPILE_CONTEXT_STACK: RefCell<Vec<Rc<CompileContext>>> = RefCell::new(Vec::new());
}
