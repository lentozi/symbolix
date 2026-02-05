use symbolix_core::semantic::semantic_ir::logic::LogicalExpression;
use symbolix_core::semantic::semantic_ir::numeric::NumericExpression;
use symbolix_core::semantic::semantic_ir::SemanticExpression;
use symbolix_core::semantic::variable::VariableType;
use symbolix_core::{context, var};

#[test]
#[should_panic]
pub fn create_without_context() {
    let _x = var!("x", VariableType::Integer, None);
}

#[test]
pub fn create_with_context() {
    context! {
        let _x = var!("x", VariableType::Integer, None);
    }
}

#[test]
pub fn numeric_variable_operations() {
    context! {
        let x = var!("x", VariableType::Integer, None);
        let y = var!("y", VariableType::Integer, None);

        let _sum = &x + &y;
        let _diff = &x - &y;
        let _prod = &x * &y;
        let _quot = &x / &y;

        assert_eq!(_sum, SemanticExpression::addition(
            SemanticExpression::numeric(NumericExpression::Variable(x.clone())), SemanticExpression::numeric(NumericExpression::Variable(y.clone()))
        ));
        assert_eq!(_diff, SemanticExpression::subtraction(
            SemanticExpression::numeric(NumericExpression::Variable(x.clone())), SemanticExpression::numeric(NumericExpression::Variable(y.clone()))
        ));
        assert_eq!(_prod, SemanticExpression::multiplication(
            SemanticExpression::numeric(NumericExpression::Variable(x.clone())), SemanticExpression::numeric(NumericExpression::Variable(y.clone()))
        ));
        assert_eq!(_quot, SemanticExpression::division(
            SemanticExpression::numeric(NumericExpression::Variable(x.clone())), SemanticExpression::numeric(NumericExpression::Variable(y.clone()))
        ));
    }
}

#[test]
pub fn logical_variable_operations() {
    context! {
        let a = var!("a", VariableType::Boolean, None);
        let b = var!("b", VariableType::Boolean, None);

        let _and = &a & &b;
        let _or = &a | &b;
        let _not_a = !&a;

        assert_eq!(_and, SemanticExpression::and(
            SemanticExpression::logical(LogicalExpression::variable(a.clone())), SemanticExpression::logical(LogicalExpression::variable(b.clone()))
        ));
        assert_eq!(_or, SemanticExpression::or(
            SemanticExpression::logical(LogicalExpression::variable(a.clone())), SemanticExpression::logical(LogicalExpression::variable(b.clone()))
        ));
        assert_eq!(_not_a, SemanticExpression::not(
            SemanticExpression::logical(LogicalExpression::variable(a.clone()))
        ));
    }
}

#[test]
#[should_panic]
pub fn mismatched_variable_operations() {
    context! {
        let x = var!("x", VariableType::Integer, None);
        let a = var!("a", VariableType::Boolean, None);

        // The following lines should cause compile-time errors if uncommented
        let _invalid_add = &x + &a;
        let _invalid_and = &x & &a;
    }
}

#[test]
pub fn variable_equality() {
    context! {
        let x1 = var!("x", VariableType::Integer, None);
        let x2 = var!("x", VariableType::Integer, None);
        let y = var!("y", VariableType::Integer, None);

        assert_eq!(x1, x2);
        assert_ne!(x1, y);
    }
}

#[test]
pub fn numeric_variable_expression_operations() {
    context! {
        let x = var!("x", VariableType::Integer, None);
        let y = var!("y", VariableType::Integer, None);

        let numeric_expr = &x + &y;
        let numeric_expr = &numeric_expr * &x;

        assert_eq!(numeric_expr, SemanticExpression::multiplication(
            SemanticExpression::addition(
                SemanticExpression::numeric(NumericExpression::Variable(x.clone())),
                SemanticExpression::numeric(NumericExpression::Variable(y.clone()))
            ),
            SemanticExpression::numeric(NumericExpression::Variable(x.clone()))
        ));
    }
}

#[test]
pub fn logical_variable_expression_operations() {
    context! {
        let a = var!("a", VariableType::Boolean, None);
        let b = var!("b", VariableType::Boolean, None);

        let logical_expr = &a | &b;
        let logical_expr = !&logical_expr;

        assert_eq!(logical_expr, SemanticExpression::not(
            SemanticExpression::or(
                SemanticExpression::logical(LogicalExpression::variable(a.clone())),
                SemanticExpression::logical(LogicalExpression::variable(b.clone()))
            )
        ));
    }
}

#[test]
pub fn numeric_expression_operations() {
    context! {
        let x = var!("x", VariableType::Integer, None);
        let y = var!("y", VariableType::Integer, None);

        let expr1 = &x + &y;
        let expr2 = &x * &y;

        let combined_expr = &expr1 - &expr2;

        assert_eq!(combined_expr, SemanticExpression::subtraction(
            SemanticExpression::addition(
                SemanticExpression::numeric(NumericExpression::Variable(x.clone())),
                SemanticExpression::numeric(NumericExpression::Variable(y.clone()))
            ),
            SemanticExpression::multiplication(
                SemanticExpression::numeric(NumericExpression::Variable(x.clone())),
                SemanticExpression::numeric(NumericExpression::Variable(y.clone()))
            )
        ));
    }
}

#[test]
pub fn logical_expression_operations() {
    context! {
        let a = var!("a", VariableType::Boolean, None);
        let b = var!("b", VariableType::Boolean, None);

        let expr1 = &a & &b;
        let expr2 = !&a;

        let combined_expr = &expr1 | &expr2;

        assert_eq!(combined_expr, SemanticExpression::or(
            SemanticExpression::and(
                SemanticExpression::logical(LogicalExpression::variable(a.clone())),
                SemanticExpression::logical(LogicalExpression::variable(b.clone()))
            ),
            SemanticExpression::not(
                SemanticExpression::logical(LogicalExpression::variable(a.clone()))
            )
        ));
    }
}

#[test]
pub fn mixed_expression_operations() {
    context! {
        let x = var!("x", VariableType::Integer, None);
        let a = var!("a", VariableType::Boolean, None);

        let numeric_expr = &x + &x;
        let logical_expr = !&a;

        // Note: Mixed operations are not directly supported; this test ensures no panic occurs
        let _ = numeric_expr;
        let _ = logical_expr;
    }
}
