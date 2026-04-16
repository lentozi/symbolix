use symbolix_core::{
    lexer::constant::Number,
    numeric_bucket,
    logical_bucket,
    semantic::{
        bucket::{LogicalBucket, NumericBucket, NumericExpressionMut},
        semantic_ir::{logic::LogicalExpression, numeric::NumericExpression},
        variable::{Variable, VariableType},
    },
};

fn numeric_var(name: &str) -> Variable {
    Variable {
        name: name.to_string(),
        var_type: VariableType::Float,
        value: None,
    }
}

fn bool_var(name: &str) -> Variable {
    Variable {
        name: name.to_string(),
        var_type: VariableType::Boolean,
        value: None,
    }
}

#[test]
fn numeric_bucket_supports_push_extend_execute_and_remove_helpers() {
    let x = NumericExpression::variable(numeric_var("x"));
    let y = NumericExpression::variable(numeric_var("y"));
    let mut bucket = NumericBucket::new();
    bucket.push(NumericExpression::constant(Number::integer(1)));
    bucket.push(x.clone());
    bucket.push(y.clone() + NumericExpression::constant(Number::integer(2)));

    assert_eq!(bucket.len(), 3);
    assert!(bucket.contains_constant());
    assert_eq!(bucket.get_constants(), vec![Number::integer(1)]);
    assert_eq!(bucket.get_non_constants().len(), 2);
    assert!(bucket.contains_one());

    let mut only_one = numeric_bucket![NumericExpression::constant(Number::integer(1))];
    assert!(only_one.contains_one());
    only_one.remove_one();
    assert_eq!(only_one.len(), 0);

    let mut only_zero = numeric_bucket![NumericExpression::constant(Number::integer(0))];
    assert!(only_zero.contains_zero());
    only_zero.remove_zero();
    assert_eq!(only_zero.len(), 0);

    let mut constants = numeric_bucket![
        NumericExpression::constant(Number::integer(2)),
        NumericExpression::constant(Number::integer(3))
    ];
    constants.execute_constant(true);
    assert_eq!(constants.get_constants(), vec![Number::integer(5)]);
    constants.execute_constant(false);
    assert_eq!(constants.get_constants(), vec![Number::integer(5)]);

    let mut more = NumericBucket::new();
    more.extend(&bucket);
    assert_eq!(more.len(), bucket.len());
    assert_eq!(bucket.without_constants().constants.len(), 0);
}

#[test]
fn numeric_bucket_iterators_intersection_and_mutation_work() {
    let x = NumericExpression::variable(numeric_var("x"));
    let expr = NumericExpression::constant(Number::integer(2)) * x.clone();
    let mut bucket = numeric_bucket![
        NumericExpression::constant(Number::integer(1)),
        x.clone(),
        expr.clone()
    ];

    let collected = bucket.iter().map(|item| item.to_string()).collect::<Vec<_>>();
    assert_eq!(collected.len(), 3);

    for item in &mut bucket {
        match item {
            NumericExpressionMut::Constant(number) => *number = Number::integer(9),
            NumericExpressionMut::Variable(variable) => variable.name = "z".to_string(),
            NumericExpressionMut::Expression(expression) => {
                *expression = NumericExpression::constant(Number::integer(7))
            }
        }
    }

    let mutated = bucket.iter().map(|item| item.to_string()).collect::<Vec<_>>();
    assert!(mutated.contains(&"9".to_string()));
    assert!(mutated.contains(&"z".to_string()));
    assert!(mutated.contains(&"7".to_string()));

    let other = numeric_bucket![
        NumericExpression::constant(Number::integer(9)),
        NumericExpression::variable(Variable {
            name: "z".to_string(),
            var_type: VariableType::Float,
            value: None,
        })
    ];
    let intersection = bucket.intersection(&other);
    assert_eq!(intersection.len(), 2);

    let removed = bucket.remove_expressions();
    assert_eq!(removed.len(), 1);
    assert_eq!(bucket.expressions.len(), 0);

    let multiples = numeric_bucket![
        NumericExpression::constant(Number::integer(2)) * NumericExpression::variable(numeric_var("a")),
        NumericExpression::constant(Number::integer(3)) * NumericExpression::variable(numeric_var("b"))
    ];
    assert!(multiples.is_all_multiples());
}

#[test]
fn logical_bucket_supports_push_extend_execute_and_iteration() {
    let flag = LogicalExpression::variable(bool_var("flag"));
    let relation = LogicalExpression::relation(
        &NumericExpression::constant(Number::integer(1)),
        &symbolix_core::lexer::symbol::Symbol::Relation(symbolix_core::lexer::symbol::Relation::LessThan),
        &NumericExpression::constant(Number::integer(2)),
    );
    let mut bucket = LogicalBucket::new();
    bucket.push(LogicalExpression::constant(true));
    bucket.push(flag.clone());
    bucket.push(relation.clone());

    assert_eq!(bucket.len(), 3);
    let collected = bucket.iter().map(|expr| expr.to_string()).collect::<Vec<_>>();
    assert_eq!(collected.len(), 3);

    let mut cloned = LogicalBucket::new();
    cloned.extend(&bucket);
    assert_eq!(cloned.len(), 3);

    let mut and_constants = logical_bucket![LogicalExpression::constant(true), LogicalExpression::constant(false)];
    and_constants.execute_constant(true);
    assert_eq!(and_constants.constants, vec![false]);
    and_constants.remove_false();
    assert_eq!(and_constants.constants.len(), 0);

    let mut or_constants = logical_bucket![LogicalExpression::constant(false), LogicalExpression::constant(true)];
    or_constants.execute_constant(false);
    assert_eq!(or_constants.constants, vec![true]);
    or_constants.remove_true();
    assert_eq!(or_constants.constants.len(), 0);

    let into_items = cloned.into_iter().map(|expr| expr.to_string()).collect::<Vec<_>>();
    assert_eq!(into_items.len(), 3);
}
