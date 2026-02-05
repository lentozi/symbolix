use ordered_float::OrderedFloat;
use symbolix_core::{
    compile::static_compile::compile_numeric,
    context,
    lexer::constant::{Constant, Number},
    semantic::{semantic_ir::SemanticExpression, variable::VariableType},
    var,
};

#[test]
fn test_compile_numeric() {
    context! {
        let mut x = var!("x", VariableType::Integer, Some(Constant::number(Number::integer(10))));
        let expr = x.to_expression();
        match expr {
            SemanticExpression::Numeric(numeric) => {
                let compiled = compile_numeric(numeric);
                assert_eq!(compiled.eval().unwrap(), Constant::Number(Number::Integer(10)));
            }
            _ => panic!("unexpected expression type"),
        }

        x.set_value(Constant::Number(Number::float(42.0)));
        let expr = x.to_expression();
        match expr {
            SemanticExpression::Numeric(numeric) => {
                let compiled = compile_numeric(numeric);
                assert_eq!(compiled.eval().unwrap(), Constant::Number(Number::Float(OrderedFloat(42.0))));
            }
            _ => panic!("unexpected expression type"),
        }
    }
}
