use crate::context::symbol_context::SymbolContext;
use crate::semantic::variable::Variable;
use std::cell::RefCell;
use std::collections::HashMap;

pub mod symbol_context;

pub type Scope = HashMap<String, Variable>;

pub struct Context {
    pub symbols: RefCell<SymbolContext>,
    pub diagnostics: RefCell<Vec<String>>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            symbols: RefCell::new(SymbolContext::new()),
            diagnostics: RefCell::new(Vec::new()),
        }
    }

    // 推入当前上下文，执行闭包，执行结束自动弹出
    pub fn with_current<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        CONTEXT_STACK.with(|stack| {
            stack.borrow_mut().push(self as *mut Context);
            let result = f(self);
            stack.borrow_mut().pop();
            result
        })
    }

    // 获取当前上下文（LTS栈顶）
    pub fn current() -> Option<&'static mut Context> {
        CONTEXT_STACK.with(|stack| {
            stack
                .borrow()
                .last()
                .map(|&ctx_ptr| unsafe { &mut *ctx_ptr })
        })
    }

    pub fn current_mut() -> Option<&'static mut Context> {
        CONTEXT_STACK.with(|stack| {
            stack
                .borrow()
                .last()
                .map(|&ctx_ptr| unsafe { &mut *ctx_ptr })
        })
    }

    pub fn report(&mut self, message: impl Into<String>) {
        self.diagnostics.borrow_mut().push(message.into());
    }

    // 进入一个新的作用域，返回 RAII guard
    pub fn scoped(&mut self) -> ScopeGuard<'_> {
        self.symbols.borrow_mut().enter_scope();
        ScopeGuard {
            symbols: &self.symbols,
        }
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
    static CONTEXT_STACK: RefCell<Vec<*mut Context>> = RefCell::new(Vec::new());
}
