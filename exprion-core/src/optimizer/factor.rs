use crate::semantic::semantic_ir::logic::LogicalExpression;
use crate::semantic::semantic_ir::numeric::NumericExpression;
use crate::semantic::semantic_ir::SemanticExpression;

pub fn factor(expr: &mut SemanticExpression) {
    match expr {
        SemanticExpression::Numeric(n) => factor_numeric(n),
        SemanticExpression::Logical(l) => factor_logic(l),
    }
}

pub fn factor_numeric(_expr: &mut NumericExpression) {}
pub fn factor_logic(_expr: &mut LogicalExpression) {}
