use exprion_core::lexer::{
    constant::Constant,
    symbol::{get_precedence, get_symbol_type, Binary, Other, Precedence, Relation, Symbol, SymbolType, Ternary, Unary},
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

#[test]
fn symbol_helpers_cover_remaining_variants_and_display_paths() {
    assert!(matches!(
        get_precedence(&Symbol::Ternary(Ternary::ConditionalElse)),
        Precedence::TERNARY
    ));
    assert!(matches!(
        get_precedence(&Symbol::Binary(Binary::LogicOr)),
        Precedence::LogicOr
    ));
    assert!(matches!(
        get_precedence(&Symbol::Other(Other::Comma)),
        Precedence::Lowest
    ));

    assert!(matches!(
        get_symbol_type(&Symbol::Unary(Unary::LogicNot)),
        SymbolType::Logical
    ));
    assert!(matches!(
        get_symbol_type(&Symbol::Ternary(Ternary::Conditional)),
        SymbolType::Conditional
    ));
    assert!(matches!(
        get_symbol_type(&Symbol::Other(Other::LeftParen)),
        SymbolType::Other
    ));

    assert_eq!(Symbol::Binary(Binary::Subtract).to_string(), "-");
    assert_eq!(Symbol::Binary(Binary::Divide).to_string(), "/");
    assert_eq!(Symbol::Binary(Binary::Modulus).to_string(), "%");
    assert_eq!(Symbol::Binary(Binary::LogicAnd).to_string(), "&&");
    assert_eq!(Symbol::Binary(Binary::LogicOr).to_string(), "||");
    assert_eq!(Symbol::Relation(Relation::Equal).to_string(), "==");
    assert_eq!(Symbol::Relation(Relation::NotEqual).to_string(), "!=");
    assert_eq!(Symbol::Relation(Relation::LessThan).to_string(), "<");
    assert_eq!(Symbol::Relation(Relation::GreaterThan).to_string(), ">");
    assert_eq!(Symbol::Other(Other::LeftParen).to_string(), "(");
    assert_eq!(Symbol::Other(Other::RightParen).to_string(), ")");
    assert_eq!(Symbol::Other(Other::Comma).to_string(), ",");
    assert_eq!(Symbol::Other(Other::Semicolon).to_string(), ";");
    assert_eq!(Symbol::Ternary(Ternary::Conditional).to_string(), "?");
    assert_eq!(Symbol::Ternary(Ternary::ConditionalElse).to_string(), ":");
    assert_eq!(Symbol::Unary(Unary::LogicNot).to_string(), "!");
}
