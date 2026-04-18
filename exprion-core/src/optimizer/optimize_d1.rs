use std::collections::{hash_map::Entry, HashMap};

use crate::{
    lexer::constant::Number,
    optimizer::optimize_term::{
        extract_addition_term, extract_logical_term, extract_multiply_term, rebuild_addition_term,
        rebuild_logical_term, rebuild_multiply_term, AdditionTerm, LogicalTerm, MultiplyTerm,
    },
    semantic::{
        bucket::{LogicalBucket, NumericBucket},
        semantic_ir::{logic::LogicalExpression, numeric::NumericExpression, SemanticExpression},
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
            let mut map: HashMap<NumericExpression, Number> = HashMap::new();
            for term in bucket
                .into_iter()
                .map(optimize_numeric_d1)
                .map(extract_addition_term)
            {
                match map.entry(term.core) {
                    Entry::Occupied(mut entry) => {
                        let updated = entry.get().clone() + term.coef;
                        entry.insert(updated);
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(term.coef);
                    }
                }
            }

            let bucket: NumericBucket = map
                .into_iter()
                .map(|(core, coef)| rebuild_addition_term(AdditionTerm::new(coef, core)))
                .collect();

            NumericExpression::Addition(bucket)
        }
        NumericExpression::Multiplication(bucket) => {
            let mut map: HashMap<NumericExpression, Number> = HashMap::new();
            for term in bucket
                .into_iter()
                .map(optimize_numeric_d1)
                .map(extract_multiply_term)
            {
                match map.entry(term.base) {
                    Entry::Occupied(mut entry) => {
                        let updated = entry.get().clone() + term.exponent;
                        entry.insert(updated);
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(term.exponent);
                    }
                }
            }

            let bucket: NumericBucket = map
                .into_iter()
                .map(|(base, exponent)| rebuild_multiply_term(MultiplyTerm { base, exponent }))
                .collect();

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
            let mut map: HashMap<LogicalExpression, bool> = HashMap::new();
            for term in bucket.into_iter().map(optimize_logic_d1).map(extract_logical_term) {
                match map.entry(term.base) {
                    Entry::Occupied(entry) => {
                        if term.is_not != *entry.get() {
                            return LogicalExpression::Constant(false);
                        }
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(term.is_not);
                    }
                }
            }

            let bucket: LogicalBucket = map
                .into_iter()
                .map(|(expr, is_not)| rebuild_logical_term(LogicalTerm::new(expr, is_not)))
                .collect();

            LogicalExpression::And(bucket)
        }
        LogicalExpression::Or(bucket) => {
            let mut map: HashMap<LogicalExpression, bool> = HashMap::new();
            for term in bucket.into_iter().map(optimize_logic_d1).map(extract_logical_term) {
                match map.entry(term.base) {
                    Entry::Occupied(entry) => {
                        if term.is_not != *entry.get() {
                            return LogicalExpression::Constant(true);
                        }
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(term.is_not);
                    }
                }
            }

            let bucket: LogicalBucket = map
                .into_iter()
                .map(|(expr, is_not)| rebuild_logical_term(LogicalTerm::new(expr, is_not)))
                .collect();

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
