use exprion_core::{semantic::{semantic_ir::SemanticExpression, variable::Variable}};

use crate::{Expr, Var};

pub trait IntoExpr {
    fn into_expr(self) -> Expr;
}

impl IntoExpr for Expr {
    fn into_expr(self) -> Expr {
        self
    }
}

impl IntoExpr for &Expr {
    fn into_expr(self) -> Expr {
        self.clone()
    }
}

impl IntoExpr for Var {
    fn into_expr(self) -> Expr {
        self.expr()
    }
}

impl IntoExpr for &Var {
    fn into_expr(self) -> Expr {
        self.expr()
    }
}

impl IntoExpr for i32 {
    fn into_expr(self) -> Expr {
        Expr::integer(self)
    }
}

impl IntoExpr for i64 {
    fn into_expr(self) -> Expr {
        Expr::float(self as f64)
    }
}

impl IntoExpr for f32 {
    fn into_expr(self) -> Expr {
        Expr::float(self as f64)
    }
}

impl IntoExpr for f64 {
    fn into_expr(self) -> Expr {
        Expr::float(self)
    }
}

impl IntoExpr for bool {
    fn into_expr(self) -> Expr {
        Expr::boolean(self)
    }
}

impl From<SemanticExpression> for Expr {
    fn from(value: SemanticExpression) -> Self {
        Expr::from_semantic(value)
    }
}

impl From<Variable> for Var {
    fn from(value: Variable) -> Self {
        Var(value)
    }
}
