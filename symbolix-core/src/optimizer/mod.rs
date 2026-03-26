mod factor;
mod flatten;
mod normalize;
mod optimize_d1;
mod optimize_term;

use factor::factor;
use normalize::normalize;
use optimize_d1::optimize_d1;

use crate::{
    optimizer::{
        factor::factor_numeric, normalize::normalize_numeric, optimize_d1::optimize_numeric_d1,
    },
    semantic::semantic_ir::{numeric::NumericExpression, SemanticExpression},
};

pub fn optimize(expr: &mut SemanticExpression) {
    normalize(expr);
    optimize_d1(expr);
    normalize(expr);
    factor(expr);
}

pub use flatten::flatten_numeric;
