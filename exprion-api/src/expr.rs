use exprion_core::{
    lexer::symbol::{Relation, Symbol},
    semantic::semantic_ir::{logic::LogicalExpression, numeric::NumericExpression, SemanticExpression},
};

use crate::IntoExpr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr(pub(crate) SemanticExpression);

impl Expr {
    pub fn from_semantic(expr: SemanticExpression) -> Self {
        Self(expr)
    }

    pub fn numeric(expr: NumericExpression) -> Self {
        Self(SemanticExpression::numeric(expr))
    }

    pub fn logical(expr: LogicalExpression) -> Self {
        Self(SemanticExpression::logical(expr))
    }

    pub fn integer(value: i32) -> Self {
        Self::numeric(NumericExpression::compatible_constant(value))
    }

    pub fn float(value: f64) -> Self {
        Self::numeric(NumericExpression::compatible_constant(value))
    }

    pub fn boolean(value: bool) -> Self {
        Self::logical(LogicalExpression::constant(value))
    }

    pub fn semantic(&self) -> &SemanticExpression {
        &self.0
    }

    pub fn into_semantic(self) -> SemanticExpression {
        self.0
    }

    pub fn as_numeric(&self) -> &NumericExpression {
        match &self.0 {
            SemanticExpression::Numeric(expr) => expr,
            SemanticExpression::Logical(_) => panic!("expected numeric expression"),
        }
    }

    pub fn as_logical(&self) -> &LogicalExpression {
        match &self.0 {
            SemanticExpression::Logical(expr) => expr,
            SemanticExpression::Numeric(_) => panic!("expected logical expression"),
        }
    }

    pub fn pow<T: IntoExpr>(&self, rhs: T) -> Self {
        let rhs = rhs.into_expr();
        Self(SemanticExpression::power(&self.0, &rhs.0))
    }

    pub fn eq_expr<T: IntoExpr>(&self, rhs: T) -> Self {
        self.relation(rhs, Relation::Equal)
    }

    pub fn ne_expr<T: IntoExpr>(&self, rhs: T) -> Self {
        self.relation(rhs, Relation::NotEqual)
    }

    pub fn lt<T: IntoExpr>(&self, rhs: T) -> Self {
        self.relation(rhs, Relation::LessThan)
    }

    pub fn le<T: IntoExpr>(&self, rhs: T) -> Self {
        self.relation(rhs, Relation::LessEqual)
    }

    pub fn gt<T: IntoExpr>(&self, rhs: T) -> Self {
        self.relation(rhs, Relation::GreaterThan)
    }

    pub fn ge<T: IntoExpr>(&self, rhs: T) -> Self {
        self.relation(rhs, Relation::GreaterEqual)
    }

    fn relation<T: IntoExpr>(&self, rhs: T, relation: Relation) -> Self {
        let rhs = rhs.into_expr();
        let left = self.as_numeric().clone();
        let right = rhs.as_numeric().clone();
        Self::logical(LogicalExpression::relation(
            &left,
            &Symbol::Relation(relation),
            &right,
        ))
    }
}
