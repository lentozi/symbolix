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
use syn::{BinOp, Block, Expr, ExprIf, Lit, LitStr, Token, UnOp};

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
    mut table: &mut HashMap<String, SemanticExpression>,
) -> (Vec<SemanticExpression>, Vec<Ident>) {
    for stmt in &block.stmts {
        match stmt {
            // let 赋值
            syn::Stmt::Local(local) => {
                // println!("{:#?}", local);
                let pat = local.pat.clone();

                let var_name = match &pat {
                    syn::Pat::Ident(ident) => ident.ident.to_string(),
                    _ => panic!("invalid pat"),
                };

                // expr 是等号右侧的元数据
                let expr_token = local.init.as_ref().unwrap().clone();

                // 右侧可能出现的：宏调用、方法调用、二元表达式、if 语句
                let expr = convert_expr(expr_token.expr.as_ref(), &mut table);

                table.insert(var_name, expr);
            }
            syn::Stmt::Item(_) => unreachable!(),
            // expr 只能作为返回值出现，可能是普通表达式，可能是元组
            syn::Stmt::Expr(expr, semi) => {
                if semi.is_some() {
                    panic!("unexpected ';'");
                }

                return match expr {
                    Expr::Tuple(tuple_expr) => {
                        let expr_list = tuple_expr
                            .elems
                            .iter()
                            .map(|x| convert_expr(x, &mut table))
                            .collect::<Vec<_>>();

                        let return_name_list = tuple_expr
                            .elems
                            .iter()
                            .enumerate()
                            .map(|(i, x)| match x {
                                Expr::Path(path) => path.path.get_ident().unwrap().clone(),
                                _ => format_ident!("_{}", i),
                            })
                            .collect::<Vec<_>>();

                        (expr_list, return_name_list)
                    }
                    _ => {
                        let expr = convert_expr(expr, &mut table);
                        (vec![expr], vec![])
                    }
                };
            }
            _ => {}
        }
    }

    panic!("block must end with return value");
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
    mut table: &mut HashMap<String, SemanticExpression>,
) -> SemanticExpression {
    match expr {
        Expr::Lit(lit_expr) => match &lit_expr.lit {
            Lit::Int(lit_int) => SemanticExpression::numeric(NumericExpression::constant(
                Number::integer(lit_int.base10_parse().expect("failed to parse lit_int")),
            )),
            Lit::Float(lit_float) => SemanticExpression::numeric(NumericExpression::constant(
                Number::float(lit_float.base10_parse().expect("failed to parse lit_float")),
            )),
            Lit::Bool(lit_bool) => {
                SemanticExpression::logical(LogicalExpression::constant(lit_bool.value))
            }
            _ => panic!("unsupported literal"),
        },
        Expr::Binary(binary_expr) => {
            let left = convert_expr(binary_expr.left.as_ref(), table);
            let right = convert_expr(binary_expr.right.as_ref(), table);
            match &binary_expr.op {
                BinOp::Add(_) => left + right,
                BinOp::Sub(_) => left - right,
                BinOp::Mul(_) => left * right,
                BinOp::Div(_) => left / right,
                BinOp::And(_) => left & right,
                BinOp::Or(_) => left | right,
                _ => {
                    let left = match left {
                        SemanticExpression::Numeric(numeric) => numeric,
                        _ => panic!("must be numeric expression"),
                    };

                    let right = match right {
                        SemanticExpression::Numeric(numeric) => numeric,
                        _ => panic!("must be numeric expression"),
                    };

                    match &binary_expr.op {
                        BinOp::Eq(_) => SemanticExpression::logical(LogicalExpression::relation(
                            &left,
                            &Symbol::Relation(Relation::Equal),
                            &right,
                        )),
                        BinOp::Ne(_) => SemanticExpression::logical(LogicalExpression::relation(
                            &left,
                            &Symbol::Relation(Relation::NotEqual),
                            &right,
                        )),
                        BinOp::Gt(_) => SemanticExpression::logical(LogicalExpression::relation(
                            &left,
                            &Symbol::Relation(Relation::GreaterThan),
                            &right,
                        )),
                        BinOp::Lt(_) => SemanticExpression::logical(LogicalExpression::relation(
                            &left,
                            &Symbol::Relation(Relation::LessThan),
                            &right,
                        )),
                        BinOp::Ge(_) => SemanticExpression::logical(LogicalExpression::relation(
                            &left,
                            &Symbol::Relation(Relation::GreaterEqual),
                            &right,
                        )),
                        BinOp::Le(_) => SemanticExpression::logical(LogicalExpression::relation(
                            &left,
                            &Symbol::Relation(Relation::LessEqual),
                            &right,
                        )),
                        _ => panic!("unsupported binary op: {:?}", binary_expr.op),
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
                            _ => panic!("unsupported type: {}", args.ty),
                        };

                        SemanticExpression::numeric(NumericExpression::variable(variable))
                    }
                    "expr" => {
                        let args: LitStr = syn::parse2(tokens).unwrap();
                        let expr_str = args.value();
                        let mut lexer = Lexer::new(&expr_str);
                        let expression = symbolix_core::parser::Parser::pratt(&mut lexer);
                        let mut analyzer = Analyzer::new();
                        analyzer.analyze_with_ctx(&expression)
                    }
                    _ => panic!("unsupported macro call: {}", ident),
                }
            } else {
                panic!("unsupported macro call: {}", mac.path.get_ident().unwrap());
            }
        }
        Expr::MethodCall(method_call_expr) => {
            let method_name = method_call_expr.method.to_string();
            let args = &method_call_expr.args;
            let receiver = method_call_expr.receiver.as_ref();

            let receiver_name = match receiver {
                Expr::Path(path) => path.path.segments[0].ident.to_string(),
                _ => panic!("method receiver must be a variable"),
            };

            let receiver_ir = table.get(&receiver_name).expect("receiver not found");

            if receiver_ir.is_equation() {
                // solve 方法
                if args.len() != 0 {
                    panic!(
                        "method call {} expects 0 argument, got {}",
                        method_name,
                        args.len()
                    );
                }

                match method_name.as_str() {
                    "solve" => {
                        let equation =
                            symbolix_core::equation::Equation::infer(receiver_ir.clone())
                                .expect("equation infer failed");
                        equation.solve_unique().expect("equation solve failed")
                    }
                    _ => panic!("unsupported method call {}", method_name),
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
                    _ => panic!("unsupported method call: {}", method_name),
                };

                let arg_ir = table.get(&arg_name).expect("argument not found");

                match (receiver_ir, arg_ir) {
                    (SemanticExpression::Numeric(receiver), SemanticExpression::Numeric(arg)) => {
                        match method_name.as_str() {
                            "equal_to" => SemanticExpression::logical(LogicalExpression::relation(
                                receiver,
                                &Symbol::Relation(Relation::Equal),
                                arg,
                            )),
                            "not_equal_to" => {
                                SemanticExpression::logical(LogicalExpression::relation(
                                    receiver,
                                    &Symbol::Relation(Relation::NotEqual),
                                    arg,
                                ))
                            }
                            "less_than" => {
                                SemanticExpression::logical(LogicalExpression::relation(
                                    receiver,
                                    &Symbol::Relation(Relation::LessThan),
                                    arg,
                                ))
                            }
                            "greater_than" => {
                                SemanticExpression::logical(LogicalExpression::relation(
                                    receiver,
                                    &Symbol::Relation(Relation::GreaterThan),
                                    arg,
                                ))
                            }
                            "less_equal" => {
                                SemanticExpression::logical(LogicalExpression::relation(
                                    receiver,
                                    &Symbol::Relation(Relation::LessEqual),
                                    arg,
                                ))
                            }
                            "greater_equal" => {
                                SemanticExpression::logical(LogicalExpression::relation(
                                    receiver,
                                    &Symbol::Relation(Relation::GreaterEqual),
                                    arg,
                                ))
                            }
                            _ => panic!("unsupported method call: {}", method_name),
                        }
                    }
                    _ => panic!("unsupported method call: {}", method_name),
                }
            }
        }
        Expr::Paren(paren_expr) => convert_expr(paren_expr.expr.as_ref(), table),
        Expr::Path(variable) => {
            if let Some(ident) = variable.path.get_ident() {
                if let Some(semantic_ir) = table.get(&ident.to_string()) {
                    semantic_ir.clone()
                } else {
                    panic!("undefined semantic expression: {}", ident);
                }
            } else {
                panic!("invalid semantic expression: {:?}", variable);
            }
        }
        Expr::Unary(unary_expr) => {
            let expr = convert_expr(unary_expr.expr.as_ref(), table);
            match unary_expr.op {
                UnOp::Not(_) => !expr,
                UnOp::Neg(_) => -expr,
                _ => panic!("unsupported unary operator: {:?}", unary_expr.op),
            }
        }
        _ => panic!("unsupported expression: {:?}", expr),
    }
}

fn handle_if(
    if_expr: &ExprIf,
    mut table: &mut HashMap<String, SemanticExpression>,
) -> SemanticExpression {
    let cond = match convert_expr(if_expr.cond.as_ref(), table) {
        SemanticExpression::Logical(logical) => logical,
        _ => panic!("expected a logical expression"),
    };

    let expr = match handle_block_in_if(&if_expr.then_branch, &mut table) {
        SemanticExpression::Numeric(numeric) => numeric,
        _ => panic!("expected numeric expression in 'if' expr"),
    };

    let else_expr = if let Some(else_branch) = &if_expr.else_branch {
        match else_branch.1.as_ref() {
            Expr::Block(block_branch) => handle_block_in_if(&block_branch.block, &mut table),
            Expr::If(if_branch) => handle_if(if_branch, &mut table),
            _ => panic!("unexpected expr after 'else'"),
        }
    } else {
        panic!("missing 'else' after 'if'");
    };

    let else_expr = match else_expr {
        SemanticExpression::Numeric(numeric) => numeric,
        _ => panic!("expected numeric expression in 'if' expr"),
    };

    let cases = vec![(cond, expr)];
    let otherwise = Some(else_expr);

    SemanticExpression::numeric(NumericExpression::piecewise(cases, otherwise))
}

fn handle_block_in_if(
    block: &Block,
    mut table: &mut HashMap<String, SemanticExpression>,
) -> SemanticExpression {
    let (expr_list, _): (Vec<SemanticExpression>, Vec<Ident>) = convert_block(&block, &mut table);

    if expr_list.len() > 1 {
        panic!("unexpected tuple in if block");
    } else if expr_list.len() == 0 {
        panic!("must return in if block");
    }

    expr_list
        .into_iter()
        .next()
        .expect("failed to run handle_block_in_if")
}
