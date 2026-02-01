use std::collections::HashMap;

use crate::{
    lexer::constant::Number,
    numeric_bucket,
    optimizer::optimize_term::{
        extract_addition_term, extract_multiply_term, rebuild_addition_term, rebuild_multiply_term,
        AdditionTerm, MultiplyTerm,
    },
    semantic::semantic_ir::{
        logic::LogicalExpression, numeric::NumericExpression, SemanticExpression,
    },
};

pub fn optimize_d1(expr: &mut SemanticExpression) {
    // 用一个占位符临时替换掉 expr 的内容，避免直接移动借用内容
    let placeholder = SemanticExpression::Numeric(NumericExpression::Constant(Number::Integer(0)));
    let taken = std::mem::replace(expr, placeholder);

    let optimized = match taken {
        SemanticExpression::Numeric(numeric) => {
            SemanticExpression::Numeric(optimize_numeric_d1(numeric))
        }
        SemanticExpression::Logical(logic) => SemanticExpression::Logical(logic),
    };

    *expr = optimized;
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

            // TODO 好像没有覆盖所有情况

            let mut bucket = numeric_bucket![];
            for expr in optimized_semantics {
                bucket.push(expr);
            }

            NumericExpression::Multiplication(bucket)
        }
        NumericExpression::Negation(n) => {
            let optimized = optimize_numeric_d1(*n);
            NumericExpression::Negation(Box::new(optimized))
        }
        NumericExpression::Power { base, exponent } => {
            let optimized_base = optimize_numeric_d1(*base);
            let optimized_exponent = optimize_numeric_d1(*exponent);
            NumericExpression::Power {
                base: Box::new(optimized_base),
                exponent: Box::new(optimized_exponent),
            }
        }
        NumericExpression::Piecewise { cases, otherwise } => {
            let optimized_cases = cases
                .into_iter()
                .map(|(condition, value)| {
                    let optimized_condition = optimize_logic_d1(condition);
                    let optimized_value = optimize_numeric_d1(value);
                    (optimized_condition, optimized_value)
                })
                .collect();
            let optimized_otherwise = match otherwise {
                Some(otherwise) => Some(Box::new(optimize_numeric_d1(*otherwise))),
                None => None,
            };
            NumericExpression::Piecewise {
                cases: optimized_cases,
                otherwise: optimized_otherwise,
            }
        }
        _ => expr,
    }
}

pub fn optimize_logic_d1(expr: LogicalExpression) -> LogicalExpression {
    expr
}
