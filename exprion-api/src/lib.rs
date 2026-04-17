mod convert;
mod expr;
mod ops;
mod scope;
mod var;

pub use convert::IntoExpr;
pub use expr::Expr;
pub use scope::scope;
pub use var::Var;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_normalizes_variable_and_expression_arithmetic() {
        scope(|| {
            let x = Var::number("x");
            let y = Var::number("y");

            let expr = &x + &y * 2.0 - 1.0;
            let rendered = expr.semantic().to_string();

            assert!(rendered.contains("x"));
            assert!(rendered.contains("y"));
            assert!(rendered.contains("2"));
            assert!(rendered.contains("1"));
        });
    }

    #[test]
    fn api_supports_relations_boolean_ops_and_pow() {
        scope(|| {
            let x = Var::number("x");
            let y = Var::number("y");

            let relation = x.gt(1.0) & y.lt(10.0);
            assert!(relation.semantic().is_logical());

            let power = (&x + 1.0).pow(&y);
            assert!(power.semantic().is_numeric());
        });
    }
}
