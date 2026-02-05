mod factor;
mod normalize;
mod optimize_d1;
mod optimize_term;

use factor::factor;
use normalize::normalize;
use optimize_d1::optimize_d1;

use crate::semantic::semantic_ir::SemanticExpression;

pub fn optimize(expr: &mut SemanticExpression) {
    normalize(expr);
    optimize_d1(expr);
    normalize(expr);
    factor(expr);
}
