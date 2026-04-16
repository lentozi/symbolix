use symbolix_core::lexer::{
    constant::Constant,
    symbol::{get_precedence, get_symbol_type, Binary, Precedence, Relation, Symbol, SymbolType, Unary},
    token::Token,
};

#[test]
fn symbol_helpers_report_precedence_and_type() {
    assert!(matches!(
        get_precedence(&Symbol::Binary(Binary::Add)),
        Precedence::Additive
    ));
    assert!(matches!(
        get_precedence(&Symbol::Binary(Binary::Power)),
        Precedence::Power
    ));
    assert!(matches!(
        get_symbol_type(&Symbol::Relation(Relation::GreaterEqual)),
        SymbolType::Relational
    ));
    assert_eq!(Symbol::Unary(Unary::Minus).to_string(), "-");
}

#[test]
fn token_constructors_preserve_payloads_and_display() {
    let constant = Token::constant(Constant::integer(7));
    let symbol = Token::symbol(Symbol::Binary(Binary::Multiply));
    let variable = Token::variable("answer".to_string());
    let invalid = Token::invalid("bad");

    assert_eq!(constant.to_string(), "Constant(7)");
    assert_eq!(symbol.to_string(), "Symbol(*)");
    assert_eq!(variable.to_string(), "Variable(answer)");
    assert!(invalid.to_string().contains("bad"));
}
