use symbolix_core::{context::testing::SymbolTable, semantic::variable::{Variable, VariableType}};

fn variable(name: &str, ty: VariableType) -> Variable {
    Variable {
        name: name.to_string(),
        var_type: ty,
        value: None,
    }
}

#[test]
fn symbol_table_supports_insert_lookup_mutation_and_scope_collection() {
    let mut table = SymbolTable::new();
    table.insert("x".to_string(), variable("x", VariableType::Float));
    assert_eq!(table.get("x").unwrap().name, "x");

    table.enter_scope();
    table.insert("x".to_string(), variable("x", VariableType::Boolean));
    table.insert("y".to_string(), variable("y", VariableType::Float));
    assert_eq!(table.get("x").unwrap().var_type, VariableType::Boolean);
    assert_eq!(table.collect().len(), 2);
    assert_eq!(table.collect_all().len(), 3);

    let found = table.get_mut("y").unwrap();
    found.name = "yy".to_string();
    assert_eq!(table.get("y").unwrap().name, "yy");
    assert!(table.get("yy").is_none());

    table.exit_scope();
    assert_eq!(table.get("x").unwrap().var_type, VariableType::Float);
    assert!(table.get("yy").is_none());
}

#[test]
fn symbol_table_handles_empty_scope_stack_gracefully() {
    let mut table = SymbolTable::new();
    table.exit_scope();
    table.exit_scope();

    assert!(table.collect().is_empty());
    assert!(table.collect_all().is_empty());
    assert!(table.get("missing").is_none());
    assert!(table.get_mut("missing").is_none());

    table.insert("ignored".to_string(), variable("ignored", VariableType::Float));
    assert!(table.get("ignored").is_none());
    assert!(table.get_mut("ignored").is_none());
}
