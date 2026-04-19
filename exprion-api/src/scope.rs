use exprion_core::{context::CompileContext, new_compile_context};

pub fn scope<R>(f: impl FnOnce() -> R) -> R {
    if let Some(ctx) = CompileContext::current() {
        ctx.with_new_scope(|_| f())
    } else {
        new_compile_context! {
            f()
        }
    }
}
