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

impl Add<Var> for f64 {
    type Output = Expr;

    fn add(self, rhs: Var) -> Self::Output {
        Expr::float(self).add(rhs)
    }
}

impl Add<&Var> for f64 {
    type Output = Expr;

    fn add(self, rhs: &Var) -> Self::Output {
        Expr::float(self).add(rhs)
    }
}

impl Add<Expr> for f64 {
    type Output = Expr;

    fn add(self, rhs: Expr) -> Self::Output {
        Expr::float(self).add(rhs)
    }
}

impl Add<&Expr> for f64 {
    type Output = Expr;

    fn add(self, rhs: &Expr) -> Self::Output {
        Expr::float(self).add(rhs)
    }
}

impl Sub<Var> for f64 {
    type Output = Expr;

    fn sub(self, rhs: Var) -> Self::Output {
        Expr::float(self).sub(rhs)
    }
}

impl Sub<&Var> for f64 {
    type Output = Expr;

    fn sub(self, rhs: &Var) -> Self::Output {
        Expr::float(self).sub(rhs)
    }
}

impl Sub<Expr> for f64 {
    type Output = Expr;

    fn sub(self, rhs: Expr) -> Self::Output {
        Expr::float(self).sub(rhs)
    }
}

impl Sub<&Expr> for f64 {
    type Output = Expr;

    fn sub(self, rhs: &Expr) -> Self::Output {
        Expr::float(self).sub(rhs)
    }
}

impl Mul<Var> for f64 {
    type Output = Expr;

    fn mul(self, rhs: Var) -> Self::Output {
        Expr::float(self).mul(rhs)
    }
}

impl Mul<&Var> for f64 {
    type Output = Expr;

    fn mul(self, rhs: &Var) -> Self::Output {
        Expr::float(self).mul(rhs)
    }
}

impl Mul<Expr> for f64 {
    type Output = Expr;

    fn mul(self, rhs: Expr) -> Self::Output {
        Expr::float(self).mul(rhs)
    }
}

impl Mul<&Expr> for f64 {
    type Output = Expr;

    fn mul(self, rhs: &Expr) -> Self::Output {
        Expr::float(self).mul(rhs)
    }
}

impl Div<Var> for f64 {
    type Output = Expr;

    fn div(self, rhs: Var) -> Self::Output {
        Expr::float(self).div(rhs)
    }
}

impl Div<&Var> for f64 {
    type Output = Expr;

    fn div(self, rhs: &Var) -> Self::Output {
        Expr::float(self).div(rhs)
    }
}

impl Div<Expr> for f64 {
    type Output = Expr;

    fn div(self, rhs: Expr) -> Self::Output {
        Expr::float(self).div(rhs)
    }
}

impl Div<&Expr> for f64 {
    type Output = Expr;

    fn div(self, rhs: &Expr) -> Self::Output {
        Expr::float(self).div(rhs)
    }
}

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
