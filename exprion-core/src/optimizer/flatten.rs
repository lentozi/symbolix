use crate::{
    numeric_bucket,
    semantic::{bucket::NumericBucket, semantic_ir::numeric::NumericExpression},
};

// TODO todo!
pub fn flatten_numeric(expr: NumericExpression) -> NumericExpression {
    match expr {
        // 对于加法，直接展开加法的每一项即可
        NumericExpression::Addition(bucket) => {
            let flattened_bucket = bucket
                .into_iter()
                .map(|expr| expr.flatten())
                .collect::<NumericBucket>();
            NumericExpression::Addition(flattened_bucket)
        }
        // 对于乘法，将表达式中的每一项相乘并且相加，最后得到只包含一个 expression 的 bucket，再将常量、变量乘进表达式
        NumericExpression::Multiplication(mut bucket) => {
            // TODO 这里必须克隆吗？
            let mut expressions = bucket
                .remove_expressions()
                .into_iter()
                .map(|expr| expr.flatten())
                .collect::<Vec<_>>();

            if expressions.len() > 1 {
                // flattened_expression 是表达式中的最后一项，不可能是 Constant 和 Variable，也不可能是 Multiplication
                // TODO 能否对类型做验证？
                let mut flattened_expression =
                    expressions.pop().expect("failed pop from expressions");

                for expr in expressions {
                    flattened_expression = flattened_expression * expr;
                }

                let mut flattened_bucket = numeric_bucket![flattened_expression];
                flattened_bucket.append(&mut bucket);
                NumericExpression::Multiplication(flattened_bucket)
            } else if expressions.len() == 1 {
                let the_only_expression = match expressions.pop() {
                    Some(expression) => expression,
                    None => panic!("failed to get the only expression"),
                };
                let mut flattened_bucket = numeric_bucket![the_only_expression];
                flattened_bucket.append(&mut bucket);
                NumericExpression::Multiplication(flattened_bucket)
            } else {
                NumericExpression::Multiplication(bucket)
            }
        }
        NumericExpression::Power { base, exponent } => NumericExpression::Power {
            base: Box::new(base.flatten()),
            exponent: Box::new(exponent.flatten()),
        },
        NumericExpression::Piecewise { cases, otherwise } => {
            let flattened_cases = cases
                .into_iter()
                .map(|(case, value)| (case, value.flatten()))
                .collect();

            let flattened_otherwise = match otherwise {
                Some(otherwise) => Some(Box::new(otherwise.flatten())),
                None => None,
            };

            NumericExpression::Piecewise {
                cases: flattened_cases,
                otherwise: flattened_otherwise,
            }
        }
        NumericExpression::Negation(numeric_expression) => {
            NumericExpression::Negation(Box::new(numeric_expression.flatten()))
        }
        _ => expr,
    }
}
