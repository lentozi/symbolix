use exprion_core::{
    semantic::variable::{Variable, VariableType},
    var,
};

use crate::{Expr, IntoExpr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Var(pub(crate) Variable);

impl Var {
    pub fn number(name: &str) -> Self {
        Self(var!(name, VariableType::Float, None))
    }

    pub fn integer(name: &str) -> Self {
        Self(var!(name, VariableType::Integer, None))
    }

    pub fn boolean(name: &str) -> Self {
        Self(var!(name, VariableType::Boolean, None))
    }

    pub fn raw(&self) -> &Variable {
        &self.0
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn var_type(&self) -> &VariableType {
        &self.0.var_type
    }

    pub fn expr(&self) -> Expr {
        Expr::from_semantic(self.0.as_expression())
    }

    pub fn pow<T: IntoExpr>(&self, rhs: T) -> Expr {
        self.expr().pow(rhs)
    }

    pub fn eq_expr<T: IntoExpr>(&self, rhs: T) -> Expr {
        self.expr().eq_expr(rhs)
    }

    pub fn ne_expr<T: IntoExpr>(&self, rhs: T) -> Expr {
        self.expr().ne_expr(rhs)
    }

    pub fn lt<T: IntoExpr>(&self, rhs: T) -> Expr {
        self.expr().lt(rhs)
    }

    pub fn le<T: IntoExpr>(&self, rhs: T) -> Expr {
        self.expr().le(rhs)
    }

    pub fn gt<T: IntoExpr>(&self, rhs: T) -> Expr {
        self.expr().gt(rhs)
    }

    pub fn ge<T: IntoExpr>(&self, rhs: T) -> Expr {
        self.expr().ge(rhs)
    }
}
