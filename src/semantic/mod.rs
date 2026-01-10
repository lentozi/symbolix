use crate::lexer::constant::{Constant, Number};
use crate::lexer::symbol::Symbol;
use crate::lexer::symbol::{Binary, Relation, Ternary, Unary};
use crate::lexer::variable::Variable;
use crate::parser::expression::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticExpression {
    Numeric(NumericExpression),
    Logical(LogicalExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumericExpression {
    Constant(Number),
    Variable(Variable),
    Negation(Box<NumericExpression>),
    Addition(Vec<NumericExpression>),
    Multiplication(Vec<NumericExpression>), // a/b = a * b^(-1)
    Power {
        base: Box<NumericExpression>,
        exponent: Box<NumericExpression>, // 是否允许任意表达式？允许：超越函数；不允许：仅允许常数指数
    },
    Piecewise {
        cases: Vec<(LogicalExpression, NumericExpression)>,
        otherwise: Option<Box<NumericExpression>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalExpression {
    Constant(bool),
    Variable(Variable),
    Not(Box<LogicalExpression>),
    And(Vec<LogicalExpression>),
    Or(Vec<LogicalExpression>),
    Relation {
        left: Box<NumericExpression>,
        operator: Symbol,
        right: Box<NumericExpression>,
    },
}

impl NumericExpression {
    pub fn constant(number: Number) -> NumericExpression {
        NumericExpression::Constant(number)
    }

    pub fn variable(variable: Variable) -> NumericExpression {
        NumericExpression::Variable(variable)
    }

    pub fn negation(expr: NumericExpression) -> NumericExpression {
        match expr {
            NumericExpression::Constant(_) |
            NumericExpression::Variable(_) => NumericExpression::Negation(Box::new(expr)),
            NumericExpression::Negation(inner) => *inner,
            NumericExpression::Addition(v) => {
                let negated_terms: Vec<NumericExpression> = v.into_iter()
                    .map(|term| NumericExpression::negation(term))
                    .collect();
                NumericExpression::Addition(negated_terms)
            }
            NumericExpression::Multiplication(v) => {
                let mut new_terms = v;
                new_terms.push(NumericExpression::Constant(Number::integer(-1)));
                NumericExpression::Multiplication(new_terms)
            }
            NumericExpression::Power { .. } => NumericExpression::Negation(Box::new(expr)),
            NumericExpression::Piecewise { cases, otherwise } => {
                let new_cases: Vec<(LogicalExpression, NumericExpression)> = cases
                    .into_iter()
                    .map(|(cond, num)| (cond, NumericExpression::negation(num)))
                    .collect();
                let new_otherwise = otherwise.map(|boxed| Box::new(NumericExpression::negation(*boxed)));
                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }
        }
    }

    pub fn addition(term1: NumericExpression, term2: NumericExpression) -> NumericExpression {
        match (term1, term2) {
            // 常量折叠
            (NumericExpression::Constant(c1), NumericExpression::Constant(c2)) => {
                NumericExpression::constant(c1 + c2)
            }

            // 常量放左侧（这里不需要 clone）
            (l, NumericExpression::Constant(c2)) => {
                NumericExpression::Addition(vec![l, NumericExpression::Constant(c2)])
            }

            // Piecewise + Piecewise
            (
                NumericExpression::Piecewise { cases: cases1, otherwise: otherwise1 },
                NumericExpression::Piecewise { cases: cases2, otherwise: otherwise2 },
            ) => {
                let mut new_cases = Vec::new();

                // 先算 otherwise × otherwise
                let new_otherwise = match (&otherwise1, &otherwise2) {
                    (Some(o1), Some(o2)) => {
                        Some(Box::new(NumericExpression::addition((**o1).clone(), (**o2).clone())))
                    }
                    _ => None,
                };

                // cases1 × cases2
                for (cond1, num1) in cases1 {
                    for (cond2, num2) in &cases2 {
                        new_cases.push((
                            LogicalExpression::And(vec![cond1.clone(), cond2.clone()]),
                            NumericExpression::addition(num1.clone(), num2.clone()),
                        ));
                    }

                    // cases1 × otherwise2
                    if let Some(ref o2) = otherwise2 {
                        new_cases.push((
                            cond1,
                            NumericExpression::addition(num1, (**o2).clone()),
                        ));
                    }
                }

                // otherwise1 × cases2
                if let Some(o1) = otherwise1 {
                    for (cond2, num2) in cases2 {
                        new_cases.push((
                            cond2,
                            NumericExpression::addition(*o1.clone(), num2),
                        ));
                    }
                }


                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }

            // Piecewise + 普通表达式
            (NumericExpression::Piecewise { cases, otherwise }, r) => {
                let new_cases = cases
                    .into_iter()
                    .map(|(cond, num)| (cond, NumericExpression::addition(num, r.clone())))
                    .collect();

                let new_otherwise = otherwise
                    .map(|o| Box::new(NumericExpression::addition(*o, r)));

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }

            // 普通表达式 + Piecewise
            (l, NumericExpression::Piecewise { cases, otherwise }) => {
                let new_cases = cases
                    .into_iter()
                    .map(|(cond, num)| (cond, NumericExpression::addition(l.clone(), num)))
                    .collect();

                let new_otherwise = otherwise
                    .map(|o| Box::new(NumericExpression::addition(l, *o)));

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }

            // Addition + Addition
            (NumericExpression::Addition(mut v1), NumericExpression::Addition(v2)) => {
                v1.extend(v2);
                NumericExpression::Addition(v1)
            }

            // Addition + r
            (NumericExpression::Addition(mut v), r) => {
                v.push(r);
                NumericExpression::Addition(v)
            }

            // l + Addition
            (l, NumericExpression::Addition(v)) => {
                let mut combined = Vec::with_capacity(v.len() + 1);
                combined.push(l);
                combined.extend(v);
                NumericExpression::Addition(combined)
            }

            // fallback
            (l, r) => NumericExpression::Addition(vec![l, r]),
        }
    }


    pub fn multiplication(term1: NumericExpression, term2: NumericExpression) -> NumericExpression {
        match (term1, term2) {
            // 常量折叠
            (NumericExpression::Constant(c1), NumericExpression::Constant(c2)) => {
                NumericExpression::Constant(c1 * c2)
            }

            // Piecewise × Piecewise
            (
                NumericExpression::Piecewise { cases: cases1, otherwise: otherwise1 },
                NumericExpression::Piecewise { cases: cases2, otherwise: otherwise2 },
            ) => {
                let mut new_cases = Vec::new();

                // 先计算 otherwise × otherwise（避免 moved value）
                let new_otherwise = match (&otherwise1, &otherwise2) {
                    (Some(o1), Some(o2)) => {
                        Some(Box::new(NumericExpression::multiplication(
                            (**o1).clone(),
                            (**o2).clone(),
                        )))
                    }
                    _ => None,
                };

                // cases1 × cases2
                for (cond1, num1) in cases1 {
                    for (cond2, num2) in &cases2 {
                        new_cases.push((
                            LogicalExpression::And(vec![cond1.clone(), cond2.clone()]),
                            NumericExpression::multiplication(num1.clone(), num2.clone()),
                        ));
                    }

                    // cases1 × otherwise2
                    if let Some(ref o2) = otherwise2 {
                        new_cases.push((
                            cond1,
                            NumericExpression::multiplication(num1, (**o2).clone()),
                        ));
                    }
                }

                // otherwise1 × cases2
                if let Some(o1) = otherwise1 {
                    for (cond2, num2) in cases2 {
                        new_cases.push((
                            cond2,
                            NumericExpression::multiplication((*o1).clone(), num2),
                        ));
                    }
                }

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }

            // Piecewise × 普通表达式
            (NumericExpression::Piecewise { cases, otherwise }, r) => {
                let new_cases = cases
                    .into_iter()
                    .map(|(cond, num)| {
                        (cond, NumericExpression::multiplication(num, r.clone()))
                    })
                    .collect();

                let new_otherwise = otherwise
                    .map(|o| Box::new(NumericExpression::multiplication(*o, r)));

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }

            // 普通表达式 × Piecewise
            (l, NumericExpression::Piecewise { cases, otherwise }) => {
                let new_cases = cases
                    .into_iter()
                    .map(|(cond, num)| {
                        (cond, NumericExpression::multiplication(l.clone(), num))
                    })
                    .collect();

                let new_otherwise = otherwise
                    .map(|o| Box::new(NumericExpression::multiplication(l, *o)));

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }

            // Multiplication × Multiplication
            (NumericExpression::Multiplication(mut v1), NumericExpression::Multiplication(v2)) => {
                v1.extend(v2);
                NumericExpression::Multiplication(v1)
            }

            // Multiplication × r
            (NumericExpression::Multiplication(mut v), r) => {
                v.push(r);
                NumericExpression::Multiplication(v)
            }

            // l × Multiplication
            (l, NumericExpression::Multiplication(v)) => {
                let mut combined = Vec::with_capacity(v.len() + 1);
                combined.push(l);
                combined.extend(v);
                NumericExpression::Multiplication(combined)
            }

            // fallback
            (l, r) => NumericExpression::Multiplication(vec![l, r]),
        }
    }

    pub fn power(base: NumericExpression, exponent: NumericExpression) -> NumericExpression {
        match base {
            NumericExpression::Power { base: b, exponent: e } => {
                let new_exponent = NumericExpression::multiplication(*e, exponent);
                NumericExpression::Power {
                    base: b,
                    exponent: Box::new(new_exponent),
                }
            }
            NumericExpression::Multiplication(v) => {
                let new_factors: Vec<NumericExpression> = v.into_iter()
                    .map(|factor| NumericExpression::power(factor, exponent.clone()))
                    .collect();
                NumericExpression::Multiplication(new_factors)
            }
            _ => NumericExpression::Power {
                base: Box::new(base),
                exponent: Box::new(exponent),
            },
        }
    }

    pub fn piecewise(cases: Vec<(LogicalExpression, NumericExpression)>, otherwise: Option<NumericExpression>) -> NumericExpression {
        NumericExpression::Piecewise {
            cases,
            otherwise: otherwise.map(Box::new),
        }
    }
}

impl LogicalExpression {
    pub fn constant(value: bool) -> LogicalExpression {
        LogicalExpression::Constant(value)
    }

    pub fn variable(variable: Variable) -> LogicalExpression {
        LogicalExpression::Variable(variable)
    }

    pub fn not(expr: LogicalExpression) -> LogicalExpression {
        match expr {
            LogicalExpression::Constant(c) => LogicalExpression::Constant(!c),
            LogicalExpression::Variable(_) => LogicalExpression::Not(Box::new(expr)),
            LogicalExpression::Not(inner) => *inner,
            LogicalExpression::And(v) => {
                let negated_terms: Vec<LogicalExpression> = v.into_iter()
                    .map(|term| LogicalExpression::not(term))
                    .collect();
                LogicalExpression::Or(negated_terms)
            }
            LogicalExpression::Or(v) => {
                let negated_terms: Vec<LogicalExpression> = v.into_iter()
                    .map(|term| LogicalExpression::not(term))
                    .collect();
                LogicalExpression::And(negated_terms)
            }
            LogicalExpression::Relation { left, operator: relation, right } => match relation {
                Symbol::Relation(Relation::Equal) => LogicalExpression::Relation {
                    left, operator: Symbol::Relation(Relation::NotEqual), right
                },
                Symbol::Relation(Relation::NotEqual) => LogicalExpression::Relation {
                    left, operator: Symbol::Relation(Relation::Equal), right
                },
                Symbol::Relation(Relation::LessThan) => LogicalExpression::Relation {
                    left, operator: Symbol::Relation(Relation::GreaterEqual), right
                },
                Symbol::Relation(Relation::GreaterThan) => LogicalExpression::Relation {
                    left, operator: Symbol::Relation(Relation::LessEqual), right
                },
                Symbol::Relation(Relation::LessEqual) => LogicalExpression::Relation {
                    left, operator: Symbol::Relation(Relation::GreaterThan), right
                },
                Symbol::Relation(Relation::GreaterEqual) => LogicalExpression::Relation {
                    left, operator: Symbol::Relation(Relation::LessThan), right
                },
                _ => panic!("unsupported relation operator: {}", relation),
            }
        }
    }

    pub fn and(expr1: LogicalExpression, expr2: LogicalExpression) -> LogicalExpression {
        match (expr1, expr2) {
            // 常量折叠
            (LogicalExpression::Constant(c1), LogicalExpression::Constant(c2)) => {
                LogicalExpression::constant(c1 && c2)
            }

            // And + And
            (LogicalExpression::And(mut v1), LogicalExpression::And(v2)) => {
                v1.extend(v2);
                LogicalExpression::And(v1)
            }

            // And + r
            (LogicalExpression::And(mut v), r) => {
                v.push(r);
                LogicalExpression::And(v)
            }

            // l + And
            (l, LogicalExpression::And(v)) => {
                let mut combined = Vec::with_capacity(v.len() + 1);
                combined.push(l);
                combined.extend(v);
                LogicalExpression::And(combined)
            }

            // fallback
            (l, r) => LogicalExpression::And(vec![l, r]),
        }
    }

    pub fn or(expr1: LogicalExpression, expr2: LogicalExpression) -> LogicalExpression {
        match (expr1, expr2) {
            // 常量折叠
            (LogicalExpression::Constant(c1), LogicalExpression::Constant(c2)) => {
                LogicalExpression::constant(c1 || c2)
            }

            // Or + Or
            (LogicalExpression::Or(mut v1), LogicalExpression::Or(v2)) => {
                v1.extend(v2);
                LogicalExpression::Or(v1)
            }

            // Or + r
            (LogicalExpression::Or(mut v), r) => {
                v.push(r);
                LogicalExpression::Or(v)
            }

            // l + Or
            (l, LogicalExpression::Or(v)) => {
                let mut combined = Vec::with_capacity(v.len() + 1);
                combined.push(l);
                combined.extend(v);
                LogicalExpression::Or(combined)
            }

            // fallback
            (l, r) => LogicalExpression::Or(vec![l, r]),
        }
    }

    pub fn relation(left: NumericExpression, operator: Symbol, right: NumericExpression) -> LogicalExpression {
        LogicalExpression::Relation {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}

fn push_left(stack: &mut Vec<Expression>, root: Expression) {
    let mut visiting = Some(root);

    loop {
        (*stack).push(visiting.clone().unwrap());
        if let Some(Expression::BinaryExpression(left, _, _)) = visiting {
            visiting = Some(*left);
            continue;
        }
        break;
    }
}

fn is_numeric(expr: &Expression) -> bool {
    match expr {
        Expression::Constant(Constant::Number(_)) | Expression::Variable(_) => true,
        Expression::Constant(Constant::Boolean(_)) => false,
        Expression::UnaryExpression(s, _) => match s {
            Symbol::Unary(Unary::Plus | Unary::Minus) => true,
            Symbol::Unary(Unary::LogicNot) => false,
            _ => panic!("unexpected symbol, expect unary operation, found {}", s)
        }
        Expression::BinaryExpression(_, s, _) => match s {
            Symbol::Binary(
                Binary::Add | Binary::Subtract | Binary::Multiply |
                Binary::Divide | Binary::Modulus | Binary::Power) => true,
            Symbol::Binary(Binary::LogicAnd | Binary::LogicOr) => false,
            _ => panic!("unexpected symbol, expect binary operation, found {}", s)
        }
        Expression::TernaryExpression(_, _, then, _, _) => is_numeric(then),
        Expression::Relation(_, _, _) => false,
        _ => panic!("unsupported expression type"),
    }
}


pub fn ast_to_semantic(expr: &Expression) -> SemanticExpression {
    let mut expr_stack: Vec<Expression> = Vec::new();
    let mut semantic_stack: Vec<SemanticExpression> = Vec::new();
    let mut last_visited: Option<Expression> = None;

    // 将左子树压入栈中
    push_left(&mut expr_stack, expr.clone());

    while expr_stack.len() > 0 {
        if last_visited.is_none() {     // 之前还没有开始访问，最先访问的一定是叶节点
            let current = expr_stack.pop().unwrap();
            visit_leaf_node(&mut semantic_stack, current.clone());
            last_visited = Some(current);
        } else {
            let expression = match expr_stack.last() {
                Some(Expression::BinaryExpression(_, operation, right)) => Some((*operation, (**right).clone())),
                Some(Expression::Relation(_, operation, right)) => Some((*operation, (**right).clone())),
                _ => None,
            };
            if expression.is_some() {       // 当前节点是二元表达式，取出操作符和右子树
                let (operation, right) = expression.unwrap();
                if right != *last_visited.as_ref().unwrap() {         // 右子树还没有被访问，继续访问右子树
                    push_left(&mut expr_stack, right);
                } else {        // 左右子树均已访问，访问根节点
                    let current = expr_stack.pop().unwrap();
                    let right_semantic = semantic_stack.pop().unwrap();
                    let left_semantic = semantic_stack.pop().unwrap();

                    match operation {
                        Symbol::Binary(Binary::Add) => match (left_semantic, right_semantic) {
                            (SemanticExpression::Numeric(left), SemanticExpression::Numeric(right)) =>
                                semantic_stack.push(SemanticExpression::Numeric(NumericExpression::addition(left, right))),
                            _ => panic!("'+' operator applied to non-numeric expressions"),
                        },
                        Symbol::Binary(Binary::Subtract) => {
                            match (left_semantic, right_semantic) {
                                (SemanticExpression::Numeric(l), SemanticExpression::Numeric(r)) =>
                                    semantic_stack.push(SemanticExpression::Numeric(NumericExpression::addition(l, NumericExpression::negation(r)))),
                                _ => panic!("'-' operator applied to mismatched or logical expression types"),
                            }
                        }
                        Symbol::Binary(Binary::Multiply) => {
                            match (left_semantic, right_semantic) {
                                (SemanticExpression::Numeric(left), SemanticExpression::Numeric(right)) =>
                                    semantic_stack.push(SemanticExpression::Numeric(NumericExpression::multiplication(left, right))),
                                _ => panic!("'*' operator applied to non-numeric expressions"),
                            }
                        }
                        Symbol::Binary(Binary::Divide) => {
                            match (left_semantic, right_semantic) {
                                (SemanticExpression::Numeric(l), SemanticExpression::Numeric(r)) =>
                                    semantic_stack.push(SemanticExpression::Numeric(NumericExpression::multiplication(
                                        l,
                                        NumericExpression::power(
                                            r,
                                            NumericExpression::constant(Number::integer(-1))
                                        )
                                    ))),
                                _ => panic!("'/' operator applied to non-numeric expressions"),
                            }
                        }
                        Symbol::Binary(Binary::Power) => {
                            match (left_semantic, right_semantic) {
                                (SemanticExpression::Numeric(base), SemanticExpression::Numeric(exponent)) =>
                                    semantic_stack.push(SemanticExpression::Numeric(NumericExpression::power(base, exponent))),
                                _ => panic!("'^' operator applied to non-numeric expressions"),
                            }
                        }
                        Symbol::Binary(Binary::LogicAnd) => {
                            match (left_semantic, right_semantic) {
                                (SemanticExpression::Logical(left), SemanticExpression::Logical(right)) =>
                                    semantic_stack.push(SemanticExpression::Logical(LogicalExpression::and(left, right))),
                                _ => panic!("'&&' operator applied to non-logical expressions"),
                            }
                        }
                        Symbol::Binary(Binary::LogicOr) => {
                            match (left_semantic, right_semantic) {
                                (SemanticExpression::Logical(left), SemanticExpression::Logical(right)) =>
                                    semantic_stack.push(SemanticExpression::Logical(LogicalExpression::or(left, right))),
                                _ => panic!("'||' operator applied to non-logical expressions"),
                            }
                        }
                        Symbol::Relation(_) => {
                            match (left_semantic, right_semantic) {
                                (SemanticExpression::Numeric(left), SemanticExpression::Numeric(right)) =>
                                    semantic_stack.push(SemanticExpression::Logical(LogicalExpression::relation(left, operation, right))),
                                _ => panic!("relation operator applied to non-numeric expressions"),
                            }
                        }
                        _ => panic!("expected binary operator, found {}", operation),
                    }
                    last_visited = Some(current);
                }
            } else {        // 当前节点不是二元表达式，直接访问根节点
                let current = expr_stack.pop().unwrap();
                visit_leaf_node(&mut semantic_stack, current.clone());
                last_visited = Some(current);
            }
        }
    }

    assert_eq!(semantic_stack.len(), 1);
    semantic_stack.pop().unwrap()
}

pub fn visit_leaf_node(stack: &mut Vec<SemanticExpression>, node: Expression) {
    match node {
        Expression::Variable(ref v) => {
            let v = (*v).clone();
            // TODO 变量的类型是如何判断的
            stack.push(SemanticExpression::Numeric(NumericExpression::variable(v)));
        }
        Expression::Constant(Constant::Number(ref n)) => {
            let n = (*n).clone();
            stack.push(SemanticExpression::Numeric(NumericExpression::constant(n)))
        }
        Expression::TernaryExpression(cond, symbol1, then, symbol2, otherwise) => {
            if symbol1 == Symbol::Ternary(Ternary::Conditional) && symbol2 == Symbol::Ternary(Ternary::Conditional) {
                // TODO 这里的迭代有无优化空间？
                let otherwise_semantic = ast_to_semantic(otherwise.as_ref());
                let then_semantic = ast_to_semantic(then.as_ref());
                let cond_semantic = ast_to_semantic(cond.as_ref());

                match (cond_semantic, then_semantic, otherwise_semantic) {
                    (SemanticExpression::Logical(c), SemanticExpression::Numeric(t), SemanticExpression::Numeric(o)) =>
                        stack.push(SemanticExpression::Numeric(NumericExpression::piecewise(
                            vec![(c, t)],
                            Some(o),
                        ))),
                    _ => panic!("ternary expression with mismatched semantic types"),
                }
            } else {
                panic!("unsupported symbols in ternary expression: {}, {}", symbol1, symbol2);
            }
        }
        Expression::UnaryExpression(symbol, expr) => {
            let expr_semantic = ast_to_semantic(&expr);
            match symbol {
                Symbol::Unary(Unary::Plus) => {
                    match expr_semantic {
                        SemanticExpression::Numeric(n) =>
                            stack.push(SemanticExpression::Numeric(n)),
                        _ => panic!("'+' operator applied to non-numeric expression"),
                    }
                }
                Symbol::Unary(Unary::Minus) => {
                    match expr_semantic {
                        SemanticExpression::Numeric(n) =>
                            stack.push(SemanticExpression::Numeric(NumericExpression::negation(n))),
                        _ => panic!("'-' operator applied to non-numeric expression"),
                    }
                }
                Symbol::Unary(Unary::LogicNot) => {
                    match expr_semantic {
                        SemanticExpression::Logical(l) =>
                            stack.push(SemanticExpression::Logical(LogicalExpression::not(l))),
                        _ => panic!("'!' operator applied to non-logical expression"),
                    }
                }
                _ => panic!("unexpected unary operator: {}", symbol),
            }
        }
        _ => panic!("expected variable or constant expression"),
    }
}
