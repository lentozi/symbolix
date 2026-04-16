use crate::{
    lexer::{
        constant::Number,
        symbol::{Relation, Symbol},
    },
    semantic::semantic_ir::{
        logic::LogicalExpression, numeric::NumericExpression, SemanticExpression,
    },
};

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
                unreachable!();
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

pub fn normalize_logic(expr: &mut LogicalExpression) {
    match expr {
        LogicalExpression::And(bucket) => {
            for expr in &mut bucket.expressions {
                normalize_logic(expr);
            }
            bucket.execute_constant(true);
            bucket.remove_true();

            if bucket.len() == 0 {
                *expr = LogicalExpression::Constant(true);
            } else if bucket.len() == 1 {
                *expr = bucket.iter().next().unwrap();
            }
        }
        LogicalExpression::Or(bucket) => {
            for expr in &mut bucket.expressions {
                normalize_logic(expr);
            }
            bucket.execute_constant(false);
            bucket.remove_false();

            if bucket.len() == 0 {
                unreachable!();
            } else if bucket.len() == 1 {
                *expr = bucket.iter().next().unwrap();
            }
        }
        LogicalExpression::Not(n) => {
            normalize_logic(n);
        }
        LogicalExpression::Relation {
            left,
            operator,
            right,
        } => {
            normalize_numeric(left);
            normalize_numeric(right);
            if let (NumericExpression::Constant(left_num), NumericExpression::Constant(right_num)) =
                (left.as_ref(), right.as_ref())
            {
                *expr = LogicalExpression::Constant(compare_relation(left_num, operator, right_num));
            }
        }
        _ => {}
    }
}

fn compare_relation(left: &Number, operator: &Symbol, right: &Number) -> bool {
    let left = left.to_float();
    let right = right.to_float();
    match operator {
        Symbol::Relation(Relation::Equal) => (left - right).abs() < 1e-9,
        Symbol::Relation(Relation::NotEqual) => (left - right).abs() >= 1e-9,
        Symbol::Relation(Relation::LessThan) => left < right,
        Symbol::Relation(Relation::GreaterThan) => left > right,
        Symbol::Relation(Relation::LessEqual) => left <= right,
        Symbol::Relation(Relation::GreaterEqual) => left >= right,
        _ => false,
    }
}
