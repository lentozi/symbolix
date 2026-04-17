use exprion_core::new_compile_context;

pub fn scope<R>(f: impl FnOnce() -> R) -> R {
    new_compile_context! {
        f()
    }
}
