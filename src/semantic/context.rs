use std::cell::RefCell;
use std::collections::HashMap;
use crate::semantic::variable::Variable;

pub type Scope = HashMap<String, Variable>;

pub struct SymbolContext {
    scopes: Vec<Scope>,
}

impl SymbolContext {
    pub fn new() -> Self {
        SymbolContext {
            scopes: Vec::new(),
        }
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
}

// RAII 守卫，用于管理作用域的进入和退出
pub struct ScopeGuard<'a> {
    symbols: &'a RefCell<SymbolContext>,
}

impl<'a> Drop for ScopeGuard<'a> {
    fn drop(&mut self) {
        self.symbols.borrow_mut().exit_scope();
    }
}

thread_local! {
    static CONTEXT_STACK: RefCell<Vec<*mut AnalysisContext>> = RefCell::new(Vec::new());
}

pub struct AnalysisContext {
    pub symbols: RefCell<SymbolContext>,
    pub diagnostics: RefCell<Vec<String>>,
}

impl AnalysisContext {
    pub fn new() -> Self {
        AnalysisContext {
            symbols: RefCell::new(SymbolContext::new()),
            diagnostics: RefCell::new(Vec::new()),
        }
    }

    // 推入当前上下文，执行闭包，执行结束自动弹出
    pub fn with_current<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        CONTEXT_STACK.with(|stack| {
            stack.borrow_mut().push(self as *mut AnalysisContext);
            let result = f(self);
            stack.borrow_mut().pop();
            result
        })
    }

    // 获取当前上下文（LTS栈顶）
    pub fn current() -> Option<&'static mut AnalysisContext> {
        CONTEXT_STACK.with(|stack| {
            stack.borrow().last().map(|&ctx_ptr| unsafe { &mut *ctx_ptr })
        })
    }

    pub fn current_mut() -> Option<&'static mut AnalysisContext> {
        CONTEXT_STACK.with(|stack| {
            stack.borrow().last().map(|&ctx_ptr| unsafe { &mut *ctx_ptr })
        })
    }

    pub fn report(&mut self, message: impl Into<String>) {
        self.diagnostics.borrow_mut().push(message.into());
    }

    // 进入一个新的作用域，返回 RAII guard
    pub fn scoped(&mut self) -> ScopeGuard<'_> {
        self.symbols.borrow_mut().enter_scope();
        ScopeGuard { symbols: &self.symbols }
    }
}
