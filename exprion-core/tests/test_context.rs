use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    rc::Rc,
};

use exprion_core::{
    context::CompileContext,
    error::{ErrorExt, ErrorKind},
    lexer::constant::Constant,
    semantic::variable::{Variable, VariableType},
};

fn variable(name: &str, var_type: VariableType) -> Variable {
    Variable {
        name_id: 0,
        name: name.to_string(),
        var_type,
        value: Some(Constant::integer(1)),
    }
}

#[test]
fn push_current_tracks_stack_and_scopes() {
    let ctx = Rc::new(CompileContext::new());
    assert!(CompileContext::current().is_none());
    assert!(CompileContext::current_mut().is_none());

    CompileContext::push_current(&ctx, |ctx| {
        assert!(CompileContext::current().is_some());
        assert!(CompileContext::current_mut().is_some());

        ctx.register_variable(variable("outer", VariableType::Integer));
        assert_eq!(ctx.collect_variables().len(), 1);

        ctx.with_new_scope(|ctx| {
            ctx.register_variable(variable("inner", VariableType::Float));
            assert!(ctx.search_variable("outer").is_some());
            assert_eq!(ctx.collect_variables().len(), 1);
            assert_eq!(ctx.collect_all_variables().len(), 2);
        });

        assert!(ctx.search_variable("inner").is_none());
        assert_eq!(ctx.collect_all_variables().len(), 1);
    });

    assert!(CompileContext::current().is_none());
}

#[test]
fn register_variable_rejects_conflicting_known_types() {
    let ctx = CompileContext::new();
    ctx.register_variable(variable("x", VariableType::Integer));

    let panic = catch_unwind(AssertUnwindSafe(|| {
        ctx.register_variable(variable("x", VariableType::Boolean));
    }));

    assert!(panic.is_err());
}

#[test]
fn register_variable_allows_unknown_and_existing_same_type() {
    let ctx = CompileContext::new();
    ctx.register_variable(variable("x", VariableType::Unknown));
    ctx.register_variable(variable("x", VariableType::Integer));
    ctx.register_variable(variable("x", VariableType::Unknown));

    let found = ctx.search_variable("x").unwrap();
    assert_eq!(found.var_type, VariableType::Unknown);
}

#[test]
fn push_error_queues_non_fatal_and_panics_on_fatal() {
    let ctx = CompileContext::new();
    ctx.push_error(ErrorExt::new(
        ErrorKind::Semantic,
        "non fatal".to_string(),
        false,
    ));

    let panic = catch_unwind(AssertUnwindSafe(|| {
        ctx.push_error(ErrorExt::new(ErrorKind::Type, "fatal".to_string(), true));
    }));

    assert!(panic.is_err());
}

#[test]
fn compile_context_exit_removes_current_stack_entry() {
    let mut ctx = std::rc::Rc::new(CompileContext::new());
    CompileContext::push_current(&ctx, |_| {
        assert!(CompileContext::current().is_some());
    });

    let inner = std::rc::Rc::new(CompileContext::new());
    CompileContext::push_current(&inner, |_| {
        assert!(CompileContext::current().is_some());
    });

    let raw = std::rc::Rc::get_mut(&mut ctx).unwrap();
    raw.exit();
    assert!(CompileContext::current().is_none());
}
