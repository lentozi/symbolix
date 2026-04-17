use std::collections::HashMap;

use crate::{
    lexer::constant::Number,
    logical_bucket, numeric_bucket,
    optimizer::optimize_term::{
        extract_addition_term, extract_logical_term, extract_multiply_term, rebuild_addition_term,
        rebuild_logical_term, rebuild_multiply_term, AdditionTerm, LogicalTerm, MultiplyTerm,
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
        SemanticExpression::Logical(logic) => SemanticExpression::Logical(optimize_logic_d1(logic)),
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
    match expr {
        LogicalExpression::And(bucket) => {
            let optimized_expressions: Vec<LogicalExpression> =
                bucket.into_iter().map(optimize_logic_d1).collect();

            let optimized_terms: Vec<LogicalTerm> = optimized_expressions
                .into_iter()
                .map(extract_logical_term)
                .collect::<Vec<LogicalTerm>>();

            let mut map: HashMap<LogicalExpression, bool> = HashMap::new();
            for term in optimized_terms {
                if map.get(&term.base).is_some() {
                    let existing_is_not = map.get(&term.base).unwrap();
                    if term.is_not == *existing_is_not {
                        map.insert(term.base, *existing_is_not);
                    } else {
                        return LogicalExpression::Constant(false);
                    }
                } else {
                    map.insert(term.base, term.is_not);
                }
            }

            let optimized_semantics: Vec<LogicalExpression> = map
                .into_iter()
                .map(|(expr, is_not)| rebuild_logical_term(LogicalTerm::new(expr, is_not)))
                .collect();

            let mut bucket = logical_bucket![];
            for expr in optimized_semantics {
                bucket.push(expr);
            }

            LogicalExpression::And(bucket)
        }
        LogicalExpression::Or(bucket) => {
            let optimized_expressions: Vec<LogicalExpression> =
                bucket.into_iter().map(optimize_logic_d1).collect();

            let optimized_terms: Vec<LogicalTerm> = optimized_expressions
                .into_iter()
                .map(extract_logical_term)
                .collect::<Vec<LogicalTerm>>();

            let mut map: HashMap<LogicalExpression, bool> = HashMap::new();
            for term in optimized_terms {
                if map.get(&term.base).is_some() {
                    let existing_is_not = map.get(&term.base).unwrap();
                    if term.is_not == *existing_is_not {
                        map.insert(term.base, *existing_is_not);
                    } else {
                        return LogicalExpression::Constant(true);
                    }
                } else {
                    map.insert(term.base, term.is_not);
                }
            }

            let optimized_semantics: Vec<LogicalExpression> = map
                .into_iter()
                .map(|(expr, is_not)| rebuild_logical_term(LogicalTerm::new(expr, is_not)))
                .collect();

            let mut bucket = logical_bucket![];
            for expr in optimized_semantics {
                bucket.push(expr);
            }

            LogicalExpression::Or(bucket)
        }
        LogicalExpression::Relation {
            left,
            operator,
            right,
        } => {
            let optimize_left = optimize_numeric_d1(*left);
            let optimize_right = optimize_numeric_d1(*right);
            LogicalExpression::Relation {
                left: Box::new(optimize_left),
                operator,
                right: Box::new(optimize_right),
            }
        }
        LogicalExpression::Not(n) => {
            let optimized_not = optimize_logic_d1(*n);
            LogicalExpression::Not(Box::new(optimized_not))
        }
        _ => expr,
    }
}
