use crate::lexer::constant::Number;
use crate::numeric_bucket;
use crate::optimizer::optimize_term::{
    extract_addition_term, extract_multiply_term, rebuild_addition_term, rebuild_multiply_term,
    AdditionTerm, MultiplyTerm,
};
use crate::semantic::semantic_ir::numeric::NumericExpression;
use crate::semantic::semantic_ir::SemanticExpression;
use std::collections::HashMap;

mod optimize_term;

pub fn optimize_d1(expr: SemanticExpression) -> SemanticExpression {
    match expr {
        SemanticExpression::Numeric(numeric) => {
            SemanticExpression::Numeric(optimize_numeric_d1(numeric))
        }
        SemanticExpression::Logical(logic) => SemanticExpression::Logical(logic),
    }
}

pub fn optimize_numeric_d1(expr: NumericExpression) -> NumericExpression {
    match expr {
        NumericExpression::Addition(bucket) => {
            let optimized_expressions: Vec<NumericExpression> =
                bucket.into_iter().map(optimize_numeric_d1).collect();

            let optimized_terms: Vec<AdditionTerm> = optimized_expressions
                .into_iter()
                .map(extract_addition_term)
                .collect::<Vec<AdditionTerm>>();

            let mut map: HashMap<NumericExpression, Number> = HashMap::new();
            for term in optimized_terms {
                if map.get(&term.core).is_some() {
                    let existing_coef = map.get(&term.core).unwrap().clone();
                    map.insert(term.core, existing_coef + term.coef);
                } else {
                    map.insert(term.core, term.coef);
                }
            }

            let optimized_semantics: Vec<NumericExpression> = map
                .into_iter()
                .map(|(core, coef)| rebuild_addition_term(AdditionTerm::new(coef, core)))
                .collect();

            let mut bucket = numeric_bucket![];
            for expr in optimized_semantics {
                bucket.push(expr);
            }

            NumericExpression::Addition(bucket)
        }
        NumericExpression::Multiplication(bucket) => {
            let optimized_expressions: Vec<NumericExpression> =
                bucket.into_iter().map(optimize_numeric_d1).collect();

            let optimized_terms: Vec<MultiplyTerm> = optimized_expressions
                .into_iter()
                .map(extract_multiply_term)
                .collect::<Vec<MultiplyTerm>>();

            let mut map: HashMap<NumericExpression, Number> = HashMap::new();
            for term in optimized_terms {
                if map.get(&term.base).is_some() {
                    let existing_coef = map.get(&term.base).unwrap().clone();
                    map.insert(term.base, existing_coef + term.exponent);
                } else {
                    map.insert(term.base, term.exponent);
                }
            }

            let optimized_semantics: Vec<NumericExpression> = map
                .into_iter()
                .map(|(base, exponent)| rebuild_multiply_term(MultiplyTerm { base, exponent }))
                .collect();

            let mut bucket = numeric_bucket![];
            for expr in optimized_semantics {
                bucket.push(expr);
            }

            NumericExpression::Multiplication(bucket)
        }
        _ => expr,
    }
}
