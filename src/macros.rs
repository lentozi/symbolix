#[macro_export]
macro_rules! context {
    { $($body:tt)* } => {{
        let mut ctx = $crate::semantic::context::AnalysisContext::new();

        ctx.with_current(|ctx| {
            let _scope = ctx.scoped();
            $($body)*
        })
    }};
}

#[macro_export]
macro_rules! with_context {
    ($ctx:ident, $($body:tt)*) => {{
        if let Some($ctx) = $crate::semantic::context::AnalysisContext::current() {
            $($body)*
        } else {
            panic!("No current AnalysisContext found");
        }
    }};
}