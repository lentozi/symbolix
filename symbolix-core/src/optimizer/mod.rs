mod factor;
mod flatten;
mod normalize;
mod optimize_d1;
mod optimize_term;

use factor::factor;
use optimize_d1::optimize_d1;

use crate::semantic::semantic_ir::SemanticExpression;

pub fn optimize(expr: &mut SemanticExpression) {
    normalize::normalize(expr);
    optimize_d1(expr);
    normalize::normalize(expr);
    factor(expr);
}

pub use flatten::flatten_numeric;
pub use normalize::{normalize, normalize_logic, normalize_numeric};

#[doc(hidden)]
pub mod testing {
    pub use super::optimize_d1::{optimize_d1, optimize_logic_d1, optimize_numeric_d1};
    pub use super::optimize_term::{
        extract_addition_term, extract_logical_term, extract_multiply_term, rebuild_addition_term,
        rebuild_logical_term, rebuild_multiply_term,
    };
}
