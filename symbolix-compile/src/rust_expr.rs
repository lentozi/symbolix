use proc_macro2::Ident;
use quote::format_ident;
use std::collections::HashMap;
use symbolix_core::lexer::constant::Number;
use symbolix_core::lexer::symbol::{Relation, Symbol};
use symbolix_core::lexer::Lexer;
use symbolix_core::semantic::semantic_ir::logic::LogicalExpression;
use symbolix_core::semantic::semantic_ir::numeric::NumericExpression;
use symbolix_core::semantic::semantic_ir::SemanticExpression;
use symbolix_core::semantic::variable::VariableType;
use symbolix_core::semantic::Analyzer;
use symbolix_core::var;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{BinOp, Block, Expr, ExprIf, Lit, LitStr, Token, UnOp};
use crate::CompileValue;

struct VarArgs {
    name: String,
    ty: String,
}

impl Parse for VarArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let ty: syn::Type = input.parse()?;

        let name = name.value();
        let ty = match ty {
            syn::Type::Path(type_path) => {
                if let Some(type_ident) = type_path.path.get_ident() {
                    type_ident.to_string()
                } else {
                    panic!("unsupported type")
                }
            }
            _ => panic!("unsupported type"),
        };

        Ok(VarArgs { name, ty })
    }
}

pub fn convert_block(
    block: &Block,
    mut table: &mut HashMap<String, CompileValue>,
) -> syn::Result<(Vec<CompileValue>, Vec<Ident>)> {
    for stmt in &block.stmts {
        match stmt {
            // let 赋值
            syn::Stmt::Local(local) => {
                // println!("{:#?}", local);
                let pat = local.pat.clone();

                let var_name = match &pat {
                    syn::Pat::Ident(ident) => ident.ident.to_string(),
                    _ => {
                        return Err(syn::Error::new_spanned(
                            &pat,
                            "symbolix! only supports identifier bindings in let statements",
                        ))
                    }
                };

                // expr 是等号右侧的元数据
                let expr_token = local.init.as_ref().unwrap().clone();

                // 右侧可能出现的：宏调用、方法调用、二元表达式、if 语句
                let expr = convert_expr(expr_token.expr.as_ref(), &mut table)?;

                table.insert(var_name, expr);
            }
            syn::Stmt::Item(_) => unreachable!(),
            // expr 只能作为返回值出现，可能是普通表达式，可能是元组
            syn::Stmt::Expr(expr, semi) => {
                if semi.is_some() {
                    return Err(syn::Error::new_spanned(
                        expr,
                        "symbolix! block must end with an expression, not a statement",
                    ));
                }

                return match expr {
                    Expr::Tuple(tuple_expr) => {
                        let expr_list = tuple_expr
                            .elems
                            .iter()
                            .map(|x| convert_expr(x, &mut table))
                            .collect::<Result<Vec<_>, _>>()?;

                        let return_name_list = tuple_expr
                            .elems
                            .iter()
                            .enumerate()
                            .map(|(i, x)| match x {
                                Expr::Path(path) => path.path.get_ident().unwrap().clone(),
                                _ => format_ident!("_{}", i),
                            })
                            .collect::<Vec<_>>();

                        Ok((expr_list, return_name_list))
                    }
                    _ => {
                        let expr = convert_expr(expr, &mut table)?;
                        Ok((vec![expr], vec![]))
                    }
                };
            }
            _ => {}
        }
    }

    Err(syn::Error::new(
        block.span(),
        "symbolix! block must end with a return expression or tuple",
    ))
}

/// 解析所有可能出现的表达式，包括：
/// 1. var! 宏调用
/// 2. expr! 宏调用
/// 3. 二元表达式
/// 4. 一元表达式
/// 5. if 语句
/// 6. 括号
/// 7. expr 的方法调用
/// 8. 单独变量
pub fn convert_expr(
    expr: &Expr,
    mut table: &mut HashMap<String, CompileValue>,
) -> syn::Result<CompileValue> {
    match expr {
        Expr::Lit(lit_expr) => Ok(CompileValue::Semantic(match &lit_expr.lit {
            Lit::Int(lit_int) => SemanticExpression::numeric(NumericExpression::constant(
                Number::integer(lit_int.base10_parse().expect("failed to parse lit_int")),
            )),
            Lit::Float(lit_float) => SemanticExpression::numeric(NumericExpression::constant(
                Number::float(lit_float.base10_parse().expect("failed to parse lit_float")),
            )),
            Lit::Bool(lit_bool) => {
                SemanticExpression::logical(LogicalExpression::constant(lit_bool.value))
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    lit_expr,
                    "symbolix! only supports integer, float, and bool literals",
                ))
            }
        })),
        Expr::Binary(binary_expr) => {
            let left = expect_semantic(convert_expr(binary_expr.left.as_ref(), table)?, binary_expr.left.as_ref())?;
            let right = expect_semantic(convert_expr(binary_expr.right.as_ref(), table)?, binary_expr.right.as_ref())?;
            match &binary_expr.op {
                BinOp::Add(_) => Ok(CompileValue::Semantic(left + right)),
                BinOp::Sub(_) => Ok(CompileValue::Semantic(left - right)),
                BinOp::Mul(_) => Ok(CompileValue::Semantic(left * right)),
                BinOp::Div(_) => Ok(CompileValue::Semantic(left / right)),
                BinOp::And(_) => Ok(CompileValue::Semantic(left & right)),
                BinOp::Or(_) => Ok(CompileValue::Semantic(left | right)),
                _ => {
                    let left = match left {
                        SemanticExpression::Numeric(numeric) => numeric,
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &binary_expr.left,
                                "comparison operators in symbolix! require numeric expressions",
                            ))
                        }
                    };

                    let right = match right {
                        SemanticExpression::Numeric(numeric) => numeric,
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &binary_expr.right,
                                "comparison operators in symbolix! require numeric expressions",
                            ))
                        }
                    };

                    match &binary_expr.op {
                        BinOp::Eq(_) => Ok(CompileValue::Semantic(SemanticExpression::logical(LogicalExpression::relation(
                            &left,
                            &Symbol::Relation(Relation::Equal),
                            &right,
                        )))),
                        BinOp::Ne(_) => Ok(CompileValue::Semantic(SemanticExpression::logical(LogicalExpression::relation(
                            &left,
                            &Symbol::Relation(Relation::NotEqual),
                            &right,
                        )))),
                        BinOp::Gt(_) => Ok(CompileValue::Semantic(SemanticExpression::logical(LogicalExpression::relation(
                            &left,
                            &Symbol::Relation(Relation::GreaterThan),
                            &right,
                        )))),
                        BinOp::Lt(_) => Ok(CompileValue::Semantic(SemanticExpression::logical(LogicalExpression::relation(
                            &left,
                            &Symbol::Relation(Relation::LessThan),
                            &right,
                        )))),
                        BinOp::Ge(_) => Ok(CompileValue::Semantic(SemanticExpression::logical(LogicalExpression::relation(
                            &left,
                            &Symbol::Relation(Relation::GreaterEqual),
                            &right,
                        )))),
                        BinOp::Le(_) => Ok(CompileValue::Semantic(SemanticExpression::logical(LogicalExpression::relation(
                            &left,
                            &Symbol::Relation(Relation::LessEqual),
                            &right,
                        )))),
                        _ => Err(syn::Error::new_spanned(
                            &binary_expr.op,
                            format!("symbolix! does not support binary operator {:?}", binary_expr.op),
                        )),
                    }
                }
            }
        }
        Expr::If(if_expr) => handle_if(if_expr, &mut table),
        Expr::Macro(macro_call) => {
            let mac = macro_call.mac.clone();

            let tokens = mac.tokens;

            if let Some(ident) = mac.path.get_ident() {
                match ident.to_string().as_str() {
                    "var" => {
                        let args: VarArgs =
                            syn::parse2(tokens).expect("failed to parse macro arguments");
                        let variable = match args.ty.as_ref() {
                            "i32" => var!(args.name.as_ref(), VariableType::Integer, None),
                            "i64" => var!(args.name.as_ref(), VariableType::Integer, None),
                            "f32" => var!(args.name.as_ref(), VariableType::Float, None),
                            "f64" => var!(args.name.as_ref(), VariableType::Float, None),
                            "bool" => var!(args.name.as_ref(), VariableType::Boolean, None),
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    macro_call,
                                    format!("var! only supports i32, i64, f32, f64, and bool, got {}", args.ty),
                                ))
                            }
                        };

                        Ok(CompileValue::Semantic(SemanticExpression::numeric(NumericExpression::variable(variable))))
                    }
                    "expr" => {
                        let args: LitStr = syn::parse2(tokens).unwrap();
                        let expr_str = args.value();
                        let mut lexer = Lexer::new(&expr_str);
                        let expression = symbolix_core::parser::Parser::pratt(&mut lexer);
                        let mut analyzer = Analyzer::new();
                        Ok(CompileValue::Semantic(analyzer.analyze_with_ctx(&expression)))
                    }
                    _ => Err(syn::Error::new_spanned(
                        macro_call,
                        format!("unsupported macro `{}` inside symbolix!", ident),
                    )),
                }
            } else {
                Err(syn::Error::new_spanned(
                    macro_call,
                    "symbolix! encountered an unsupported macro call",
                ))
            }
        }
        Expr::MethodCall(method_call_expr) => {
            let method_name = method_call_expr.method.to_string();
            let args = &method_call_expr.args;
            let receiver = method_call_expr.receiver.as_ref();

            let receiver_name = match receiver {
                Expr::Path(path) => path.path.segments[0].ident.to_string(),
                _ => {
                    return Err(syn::Error::new_spanned(
                        receiver,
                        "method receiver in symbolix! must be a named binding",
                    ))
                }
            };

            let receiver_ir = table.get(&receiver_name).expect("receiver not found");

            if is_equation(receiver_ir) {
                // solve 方法
                if args.len() > 1 {
                    panic!(
                        "method call {} expects 0 or 1 argument, got {}",
                        method_name,
                        args.len()
                    );
                }

                match method_name.as_str() {
                    "solve" => {
                        let equation = if let Some(arg) = args.first() {
                            let arg_name = match arg {
                                Expr::Path(path) => path.path.segments[0].ident.to_string(),
                                _ => {
                                    return Err(syn::Error::new_spanned(
                                        arg,
                                        "equation.solve(arg) requires `arg` to be a named variable",
                                    ))
                                }
                            };
                            let arg_ir = table.get(&arg_name).expect("solve target not found");
                            let solve_for = match arg_ir {
                                CompileValue::Semantic(SemanticExpression::Numeric(NumericExpression::Variable(variable))) => {
                                    variable.clone()
                                }
                                _ => {
                                    return Err(syn::Error::new_spanned(
                                        arg,
                                        "equation.solve(arg) requires `arg` to be a numeric variable binding",
                                    ))
                                }
                            };

                            symbolix_core::equation::Equation::new(
                                expect_semantic(receiver_ir.clone(), receiver)?,
                                solve_for,
                            )
                                .expect("equation build failed")
                        } else {
                            symbolix_core::equation::Equation::infer(
                                expect_semantic(receiver_ir.clone(), receiver)?,
                            )
                                .expect("equation infer failed")
                        };

                        Ok(CompileValue::SolutionSet(equation.solve().expect("equation solve failed")))
                    }
                    _ => Err(syn::Error::new_spanned(
                        method_call_expr,
                        format!("unsupported equation method `{}` in symbolix!", method_name),
                    )),
                }
            } else {
                if args.len() != 1 {
                    panic!(
                        "method call {} expects 1 argument, got {}",
                        method_name,
                        args.len()
                    );
                }

                // 取出第一个参数
                let arg = args.first().unwrap();

                let arg_name: String = match arg {
                    Expr::Path(path) => path.path.segments[0].ident.to_string(),
                    _ => {
                        return Err(syn::Error::new_spanned(
                            arg,
                            format!("method `{}` in symbolix! requires a named variable argument", method_name),
                        ))
                    }
                };

                let arg_ir = expect_semantic(table.get(&arg_name).expect("argument not found").clone(), arg)?;

                match (expect_semantic(receiver_ir.clone(), receiver)?, arg_ir) {
                    (SemanticExpression::Numeric(receiver), SemanticExpression::Numeric(arg)) => {
                        match method_name.as_str() {
                            "equal_to" => Ok(CompileValue::Semantic(SemanticExpression::logical(LogicalExpression::relation(
                                &receiver,
                                &Symbol::Relation(Relation::Equal),
                                &arg,
                            )))),
                            "not_equal_to" => {
                                Ok(CompileValue::Semantic(SemanticExpression::logical(LogicalExpression::relation(
                                    &receiver,
                                    &Symbol::Relation(Relation::NotEqual),
                                    &arg,
                                ))))
                            }
                            "less_than" => {
                                Ok(CompileValue::Semantic(SemanticExpression::logical(LogicalExpression::relation(
                                    &receiver,
                                    &Symbol::Relation(Relation::LessThan),
                                    &arg,
                                ))))
                            }
                            "greater_than" => {
                                Ok(CompileValue::Semantic(SemanticExpression::logical(LogicalExpression::relation(
                                    &receiver,
                                    &Symbol::Relation(Relation::GreaterThan),
                                    &arg,
                                ))))
                            }
                            "less_equal" => {
                                Ok(CompileValue::Semantic(SemanticExpression::logical(LogicalExpression::relation(
                                    &receiver,
                                    &Symbol::Relation(Relation::LessEqual),
                                    &arg,
                                ))))
                            }
                            "greater_equal" => {
                                Ok(CompileValue::Semantic(SemanticExpression::logical(LogicalExpression::relation(
                                    &receiver,
                                    &Symbol::Relation(Relation::GreaterEqual),
                                    &arg,
                                ))))
                            }
                            _ => Err(syn::Error::new_spanned(
                                method_call_expr,
                                format!("unsupported numeric method `{}` in symbolix!", method_name),
                            )),
                        }
                    }
                    _ => Err(syn::Error::new_spanned(
                        method_call_expr,
                        format!("method `{}` in symbolix! is only supported for numeric expressions", method_name),
                    )),
                }
            }
        }
        Expr::Paren(paren_expr) => convert_expr(paren_expr.expr.as_ref(), table),
        Expr::Path(variable) => {
            if let Some(ident) = variable.path.get_ident() {
                if let Some(semantic_ir) = table.get(&ident.to_string()) {
                    Ok(semantic_ir.clone())
                } else {
                    Err(syn::Error::new_spanned(
                        variable,
                        format!("undefined binding `{}` in symbolix! block", ident),
                    ))
                }
            } else {
                Err(syn::Error::new_spanned(
                    variable,
                    "symbolix! encountered an unsupported path expression",
                ))
            }
        }
        Expr::Unary(unary_expr) => {
            let expr = expect_semantic(convert_expr(unary_expr.expr.as_ref(), table)?, unary_expr.expr.as_ref())?;
            match unary_expr.op {
                UnOp::Not(_) => Ok(CompileValue::Semantic(!expr)),
                UnOp::Neg(_) => Ok(CompileValue::Semantic(-expr)),
                _ => Err(syn::Error::new_spanned(
                    unary_expr,
                    format!("symbolix! does not support unary operator {:?}", unary_expr.op),
                )),
            }
        }
        _ => Err(syn::Error::new_spanned(
            expr,
            format!("symbolix! encountered an unsupported expression shape: {:?}", expr),
        )),
    }
}

fn handle_if(
    if_expr: &ExprIf,
    mut table: &mut HashMap<String, CompileValue>,
) -> syn::Result<CompileValue> {
    let cond = match expect_semantic(convert_expr(if_expr.cond.as_ref(), table)?, if_expr.cond.as_ref())? {
        SemanticExpression::Logical(logical) => logical,
        _ => {
            return Err(syn::Error::new_spanned(
                &if_expr.cond,
                "if condition inside symbolix! must be a logical expression",
            ))
        }
    };

    let expr = match expect_semantic(
        handle_block_in_if(&if_expr.then_branch, &mut table)?,
        &if_expr.then_branch,
    )? {
        SemanticExpression::Numeric(numeric) => numeric,
        _ => {
            return Err(syn::Error::new_spanned(
                &if_expr.then_branch,
                "if branch inside symbolix! must produce a numeric expression",
            ))
        }
    };

    let else_expr = if let Some(else_branch) = &if_expr.else_branch {
        match else_branch.1.as_ref() {
            Expr::Block(block_branch) => handle_block_in_if(&block_branch.block, &mut table),
            Expr::If(if_branch) => handle_if(if_branch, &mut table),
            _ => {
                return Err(syn::Error::new_spanned(
                    else_branch.1.as_ref(),
                    "else branch inside symbolix! must be a block or nested if",
                ))
            }
        }
    } else {
        return Err(syn::Error::new_spanned(
            if_expr,
            "symbolix! requires an else branch for every if expression",
        ));
    };

    let else_expr = match expect_semantic(else_expr?, if_expr)? {
        SemanticExpression::Numeric(numeric) => numeric,
        _ => {
            return Err(syn::Error::new_spanned(
                if_expr,
                "else branch inside symbolix! must produce a numeric expression",
            ))
        }
    };

    let cases = vec![(cond, expr)];
    let otherwise = Some(else_expr);

    Ok(CompileValue::Semantic(SemanticExpression::numeric(NumericExpression::piecewise(cases, otherwise))))
}

fn handle_block_in_if(
    block: &Block,
    mut table: &mut HashMap<String, CompileValue>,
) -> syn::Result<CompileValue> {
    let (expr_list, _): (Vec<CompileValue>, Vec<Ident>) = convert_block(&block, &mut table)?;

    if expr_list.len() > 1 {
        return Err(syn::Error::new_spanned(
            block,
            "if blocks inside symbolix! cannot return tuples",
        ));
    } else if expr_list.len() == 0 {
        return Err(syn::Error::new_spanned(
            block,
            "if blocks inside symbolix! must end with a return expression",
        ));
    }

    Ok(expr_list
        .into_iter()
        .next()
        .expect("failed to run handle_block_in_if"))
}

fn expect_semantic(value: CompileValue, span: &impl Spanned) -> syn::Result<SemanticExpression> {
    match value {
        CompileValue::Semantic(expr) => Ok(expr),
        CompileValue::SolutionSet(_) => Err(syn::Error::new(
            span.span(),
            "solution sets cannot participate in expression arithmetic inside symbolix!",
        )),
    }
}

fn is_equation(value: &CompileValue) -> bool {
    matches!(
        value,
        CompileValue::Semantic(SemanticExpression::Logical(LogicalExpression::Relation {
            operator: Symbol::Relation(Relation::Equal),
            ..
        }))
    )
}

