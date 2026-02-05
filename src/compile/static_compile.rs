use crate::{
    compile::EvalFn,
    error::Error,
    lexer::constant::Constant,
    semantic::{
        semantic_ir::{numeric::NumericExpression, SemanticExpression},
        variable::VariableType,
    },
    with_context,
};

pub struct StaticRule<F>
where
    F: Fn() -> Result<Constant, Error>,
{
    f: F,
}

impl<F> EvalFn for StaticRule<F>
where
    F: Fn() -> Result<Constant, Error>,
{
    fn eval(&self) -> Result<Constant, Error> {
        (self.f)()
    }
}

pub fn compile_numeric(expr: NumericExpression) -> Box<dyn EvalFn> {
    match expr {
        NumericExpression::Constant(c) => {
            let f = move || Ok(Constant::Number(c.clone()));
            Box::new(StaticRule { f })
        }
        NumericExpression::Variable(v) => {
            let f = move || {
                match with_context!(ctx, ctx.symbols.borrow().find(&v.name).or(None)) {
                    Some(var) => {
                        // 检查变量类型是否为数值类型
                        if var.var_type == VariableType::Boolean
                            || var.var_type == VariableType::Unknown
                        {
                            Err(Error::semantic_error(&format!(
                                "variable '{}' is not numeric",
                                var.name
                            )))
                        } else {
                            let value = var.get_value().expect("value not set");
                            match value {
                                Constant::Number(n) => Ok(Constant::Number(n)),
                                _ => panic!("unexpected type for variable {}", var.name),
                            }
                        }
                    }
                    None => Err(Error::semantic_error(&format!(
                        "variable '{}' not declared",
                        v.name
                    ))),
                }
            };
            Box::new(StaticRule { f })
        }
        _ => {
            let f = move || Err(Error::semantic_error("unexpect expression"));
            Box::new(StaticRule { f })
        }
    }
}
