use crate::lexer::constant::Number;
use crate::numeric_bucket;
use crate::optimizer::optimize_term::{
    extract_addition_term, extract_multiply_term, rebuild_addition_term, rebuild_multiply_term,
    AdditionTerm, MultiplyTerm,
};
use crate::semantic::semantic_ir::logic::LogicalExpression;
use crate::semantic::semantic_ir::numeric::NumericExpression;
use crate::semantic::semantic_ir::SemanticExpression;
use std::collections::HashMap;

mod optimize_term;

pub fn normalize(expr: &mut SemanticExpression) {
    match expr {
        SemanticExpression::Numeric(numeric) => {
            normalize_numeric(numeric);
        }
        SemanticExpression::Logical(logic) => {
            normalize_logic(logic);
        }
    }
}

pub fn normalize_numeric(expr: &mut NumericExpression) {
    match expr {
        NumericExpression::Addition(bucket) => {
            for expr in &mut bucket.expressions {
                normalize_numeric(expr);
            }
            bucket.execute_constant(true);
            bucket.remove_zero();

            if bucket.len() == 0 {
                *expr = NumericExpression::Constant(Number::Integer(0));
            } else if bucket.len() == 1 {
                *expr = bucket.iter().next().unwrap();
            }
        }
        NumericExpression::Multiplication(bucket) => {
            for expr in &mut bucket.expressions {
                normalize_numeric(expr);
            }
            bucket.execute_constant(false);
            bucket.remove_one();

            if bucket.len() == 0 {
                *expr = NumericExpression::Constant(Number::Integer(0));
            } else if bucket.len() == 1 {
                *expr = bucket.iter().next().unwrap();
            } else if bucket.contains_zero() {
                *expr = NumericExpression::Constant(Number::Integer(0));
            }
        }
        NumericExpression::Negation(inner) => {
            normalize_numeric(inner);
        }
        NumericExpression::Power { base, exponent } => {
            normalize_numeric(base);
            normalize_numeric(exponent);
        }
        NumericExpression::Piecewise { cases, otherwise } => {
            for (cond, num) in cases {
                normalize_logic(cond);
                normalize_numeric(num);
            }
            if let Some(otherwise) = otherwise {
                normalize_numeric(otherwise);
            }
        }
        _ => {}
    }
}

pub fn normalize_logic(expr: &mut LogicalExpression) {}

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
