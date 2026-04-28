use std::ops::{Add, BitAnd, BitOr, Div, Mul, Neg, Not, Sub};

use crate::{Expr, IntoExpr, Var};

macro_rules! impl_binary_expr_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<T: IntoExpr> $trait<T> for Expr {
            type Output = Expr;

            fn $method(self, rhs: T) -> Self::Output {
                let rhs = rhs.into_expr();
                Expr(self.0 $op rhs.0)
            }
        }

        impl<T: IntoExpr> $trait<T> for Var {
            type Output = Expr;

            fn $method(self, rhs: T) -> Self::Output {
                self.expr().$method(rhs)
            }
        }

        impl<T: IntoExpr> $trait<T> for &Expr {
            type Output = Expr;

            fn $method(self, rhs: T) -> Self::Output {
                let rhs = rhs.into_expr();
                Expr(self.0.clone() $op rhs.0)
            }
        }

        impl<T: IntoExpr> $trait<T> for &Var {
            type Output = Expr;

            fn $method(self, rhs: T) -> Self::Output {
                self.expr().$method(rhs)
            }
        }
    };
}

impl_binary_expr_op!(Add, add, +);
impl_binary_expr_op!(Sub, sub, -);
impl_binary_expr_op!(Mul, mul, *);
impl_binary_expr_op!(Div, div, /);
impl_binary_expr_op!(BitAnd, bitand, &);
impl_binary_expr_op!(BitOr, bitor, |);

macro_rules! impl_scalar_left_expr_op {
    ($lhs:ty, $ctor:ident, $cast:expr, $trait:ident, $method:ident) => {
        impl $trait<Var> for $lhs {
            type Output = Expr;

            fn $method(self, rhs: Var) -> Self::Output {
                let lhs_expr = Expr::$ctor($cast);
                lhs_expr.$method(rhs)
            }
        }

        impl $trait<&Var> for $lhs {
            type Output = Expr;

            fn $method(self, rhs: &Var) -> Self::Output {
                let lhs_expr = Expr::$ctor($cast);
                lhs_expr.$method(rhs)
            }
        }

        impl $trait<Expr> for $lhs {
            type Output = Expr;

            fn $method(self, rhs: Expr) -> Self::Output {
                let lhs_expr = Expr::$ctor($cast);
                lhs_expr.$method(rhs)
            }
        }

        impl $trait<&Expr> for $lhs {
            type Output = Expr;

            fn $method(self, rhs: &Expr) -> Self::Output {
                let lhs_expr = Expr::$ctor($cast);
                lhs_expr.$method(rhs)
            }
        }
    };
}

impl_scalar_left_expr_op!(f64, float, self, Add, add);
impl_scalar_left_expr_op!(f64, float, self, Sub, sub);
impl_scalar_left_expr_op!(f64, float, self, Mul, mul);
impl_scalar_left_expr_op!(f64, float, self, Div, div);

impl_scalar_left_expr_op!(f32, float, self as f64, Add, add);
impl_scalar_left_expr_op!(f32, float, self as f64, Sub, sub);
impl_scalar_left_expr_op!(f32, float, self as f64, Mul, mul);
impl_scalar_left_expr_op!(f32, float, self as f64, Div, div);

impl_scalar_left_expr_op!(i64, float, self as f64, Add, add);
impl_scalar_left_expr_op!(i64, float, self as f64, Sub, sub);
impl_scalar_left_expr_op!(i64, float, self as f64, Mul, mul);
impl_scalar_left_expr_op!(i64, float, self as f64, Div, div);

impl_scalar_left_expr_op!(i32, integer, self, Add, add);
impl_scalar_left_expr_op!(i32, integer, self, Sub, sub);
impl_scalar_left_expr_op!(i32, integer, self, Mul, mul);
impl_scalar_left_expr_op!(i32, integer, self, Div, div);

impl Neg for Expr {
    type Output = Expr;

    fn neg(self) -> Self::Output {
        Expr(-self.0)
    }
}

impl Neg for Var {
    type Output = Expr;

    fn neg(self) -> Self::Output {
        -self.expr()
    }
}

impl Neg for &Expr {
    type Output = Expr;

    fn neg(self) -> Self::Output {
        Expr(-self.0.clone())
    }
}

impl Neg for &Var {
    type Output = Expr;

    fn neg(self) -> Self::Output {
        -self.expr()
    }
}

impl Not for Expr {
    type Output = Expr;

    fn not(self) -> Self::Output {
        Expr(!self.0)
    }
}

impl Not for Var {
    type Output = Expr;

    fn not(self) -> Self::Output {
        !self.expr()
    }
}

impl Not for &Expr {
    type Output = Expr;

    fn not(self) -> Self::Output {
        Expr(!self.0.clone())
    }
}

impl Not for &Var {
    type Output = Expr;

    fn not(self) -> Self::Output {
        !self.expr()
    }
}
