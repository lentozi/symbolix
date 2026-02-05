use crate::lexer::constant::Constant;
use crate::semantic::semantic_ir::logic::LogicalExpression;
use crate::semantic::semantic_ir::numeric::NumericExpression;
use crate::semantic::semantic_ir::SemanticExpression;
use crate::{
    impl_var_binary_operation, impl_var_expr_binary_operation, impl_var_logic_operation,
    impl_var_numeric_operation, impl_var_unary_operation, with_context,
};
use log::warn;
use std::fmt;
use std::ops::{Add, BitAnd, BitOr, Div, Mul, Neg, Not, Sub};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VariableType {
    Integer,
    Float,
    Fraction,
    Boolean,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Variable {
    pub name: String,
    pub var_type: VariableType,
    pub value: Option<Constant>,
}

impl Variable {
    pub fn new(name: &str, var_type: VariableType, init_val: Option<Constant>) -> Variable {
        with_context!(ctx, {
            let symbols = &mut ctx.symbols.borrow_mut();
            match symbols.find(name) {
                Some(res) => {
                    warn!("variable '{}' already exists in the current context", name);
                    res
                }
                None => {
                    // Variable does not exist in the current context, proceed to create a new one
                    let variable = Variable {
                        name: name.to_string(),
                        var_type: var_type.clone(),
                        value: init_val,
                    };
                    symbols.insert(variable.clone());
                    variable
                }
            }
        })
    }

    pub fn to_expression(&self) -> SemanticExpression {
        match self.var_type {
            VariableType::Integer | VariableType::Float | VariableType::Fraction => {
                SemanticExpression::numeric(NumericExpression::variable(self.clone()))
            }
            VariableType::Boolean => {
                SemanticExpression::logical(LogicalExpression::variable(self.clone()))
            }
            VariableType::Unknown => {
                panic!("cannot convert variable with unknown type to expression");
            }
        }
    }

    pub fn get_value(&self) -> Option<Constant> {
        self.value.clone()
    }

    pub fn set_value(&mut self, value: Constant) {
        self.value = Some(value.clone());
        let mut symbols = with_context!(ctx, ctx.symbols.borrow_mut());
        if let Some(symbol) = symbols.find_mut(&self.name) {
            symbol.value = Some(value);
        }
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Variable {
    pub fn addition(self, other: Variable) -> SemanticExpression {
        if self.var_type == VariableType::Boolean || other.var_type == VariableType::Boolean {
            panic!("cannot add boolean variables");
        }
        if self.var_type == VariableType::Unknown || other.var_type == VariableType::Unknown {
            panic!("cannot add variables with unknown type");
        }
        SemanticExpression::numeric(NumericExpression::addition(
            NumericExpression::variable(self),
            NumericExpression::variable(other),
        ))
    }

    pub fn add_expr(self, other: SemanticExpression) -> SemanticExpression {
        if self.var_type == VariableType::Boolean {
            panic!("cannot add boolean variable");
        }
        if self.var_type == VariableType::Unknown {
            panic!("cannot add variable with unknown type");
        }
        match other {
            SemanticExpression::Numeric(num_expr) => SemanticExpression::numeric(
                NumericExpression::addition(NumericExpression::variable(self), num_expr),
            ),
            _ => panic!("addition is only defined for numeric expressions"),
        }
    }

    pub fn subtraction(self, other: Variable) -> SemanticExpression {
        if self.var_type == VariableType::Boolean || other.var_type == VariableType::Boolean {
            panic!("cannot subtract boolean variables");
        }
        if self.var_type == VariableType::Unknown || other.var_type == VariableType::Unknown {
            panic!("cannot subtract variables with unknown type");
        }
        SemanticExpression::numeric(NumericExpression::subtraction(
            NumericExpression::variable(self),
            NumericExpression::variable(other),
        ))
    }

    pub fn sub_expr(self, other: SemanticExpression) -> SemanticExpression {
        if self.var_type == VariableType::Boolean {
            panic!("cannot subtract boolean variable");
        }
        if self.var_type == VariableType::Unknown {
            panic!("cannot subtract variable with unknown type");
        }
        match other {
            SemanticExpression::Numeric(num_expr) => SemanticExpression::numeric(
                NumericExpression::subtraction(NumericExpression::variable(self), num_expr),
            ),
            _ => panic!("subtraction is only defined for numeric expressions"),
        }
    }

    pub fn multiplication(self, other: Variable) -> SemanticExpression {
        if self.var_type == VariableType::Boolean || other.var_type == VariableType::Boolean {
            panic!("cannot multiply boolean variables");
        }
        if self.var_type == VariableType::Unknown || other.var_type == VariableType::Unknown {
            panic!("cannot multiply variables with unknown type");
        }
        SemanticExpression::numeric(NumericExpression::multiplication(
            NumericExpression::variable(self),
            NumericExpression::variable(other),
        ))
    }

    pub fn mul_expr(self, other: SemanticExpression) -> SemanticExpression {
        if self.var_type == VariableType::Boolean {
            panic!("cannot multiply boolean variable");
        }
        if self.var_type == VariableType::Unknown {
            panic!("cannot multiply variable with unknown type");
        }
        match other {
            SemanticExpression::Numeric(num_expr) => SemanticExpression::numeric(
                NumericExpression::multiplication(NumericExpression::variable(self), num_expr),
            ),
            _ => panic!("multiplication is only defined for numeric expressions"),
        }
    }

    pub fn division(self, other: Variable) -> SemanticExpression {
        if self.var_type == VariableType::Boolean || other.var_type == VariableType::Boolean {
            panic!("cannot divide boolean variables");
        }
        if self.var_type == VariableType::Unknown || other.var_type == VariableType::Unknown {
            panic!("cannot divide variables with unknown type");
        }
        SemanticExpression::numeric(NumericExpression::division(
            NumericExpression::variable(self),
            NumericExpression::variable(other),
        ))
    }

    pub fn div_expr(self, other: SemanticExpression) -> SemanticExpression {
        if self.var_type == VariableType::Boolean {
            panic!("cannot divide boolean variable");
        }
        if self.var_type == VariableType::Unknown {
            panic!("cannot divide variable with unknown type");
        }
        match other {
            SemanticExpression::Numeric(num_expr) => SemanticExpression::numeric(
                NumericExpression::division(NumericExpression::variable(self), num_expr),
            ),
            _ => panic!("division is only defined for numeric expressions"),
        }
    }

    pub fn pow(self, exponent: Variable) -> SemanticExpression {
        if self.var_type == VariableType::Boolean || exponent.var_type == VariableType::Boolean {
            panic!("cannot exponentiate boolean variables");
        }
        if self.var_type == VariableType::Unknown || exponent.var_type == VariableType::Unknown {
            panic!("cannot exponentiate variables with unknown type");
        }
        SemanticExpression::numeric(NumericExpression::power(
            NumericExpression::variable(self),
            NumericExpression::variable(exponent),
        ))
    }

    pub fn pow_expr(self, other: SemanticExpression) -> SemanticExpression {
        if self.var_type == VariableType::Boolean {
            panic!("cannot exponentiate boolean variable");
        }
        if self.var_type == VariableType::Unknown {
            panic!("cannot exponentiate variable with unknown type");
        }
        match other {
            SemanticExpression::Numeric(num_expr) => SemanticExpression::numeric(
                NumericExpression::power(NumericExpression::variable(self), num_expr),
            ),
            _ => panic!("power is only defined for numeric expressions"),
        }
    }

    pub fn negation(self) -> SemanticExpression {
        if self.var_type == VariableType::Boolean {
            panic!("cannot negate boolean variable");
        }
        if self.var_type == VariableType::Unknown {
            panic!("cannot negate variable with unknown type");
        }
        SemanticExpression::numeric(NumericExpression::negation(NumericExpression::variable(
            self,
        )))
    }

    pub fn and(self, other: Variable) -> SemanticExpression {
        if self.var_type != VariableType::Boolean || other.var_type != VariableType::Boolean {
            panic!("AND operation is only defined for boolean variables");
        }
        SemanticExpression::logical(LogicalExpression::and(
            LogicalExpression::variable(self),
            LogicalExpression::variable(other),
        ))
    }

    pub fn and_expr(self, other: SemanticExpression) -> SemanticExpression {
        if self.var_type != VariableType::Boolean {
            panic!("AND operation is only defined for boolean variables");
        }
        match other {
            SemanticExpression::Logical(log_expr) => SemanticExpression::logical(
                LogicalExpression::and(LogicalExpression::variable(self), log_expr),
            ),
            _ => panic!("AND operation is only defined for logical expressions"),
        }
    }

    pub fn or(self, other: Variable) -> SemanticExpression {
        if self.var_type != VariableType::Boolean || other.var_type != VariableType::Boolean {
            panic!("OR operation is only defined for boolean variables");
        }
        SemanticExpression::logical(LogicalExpression::or(
            LogicalExpression::variable(self),
            LogicalExpression::variable(other),
        ))
    }

    pub fn or_expr(self, other: SemanticExpression) -> SemanticExpression {
        if self.var_type != VariableType::Boolean {
            panic!("OR operation is only defined for boolean variables");
        }
        match other {
            SemanticExpression::Logical(log_expr) => SemanticExpression::logical(
                LogicalExpression::or(LogicalExpression::variable(self), log_expr),
            ),
            _ => panic!("OR operation is only defined for logical expressions"),
        }
    }

    pub fn not(self) -> SemanticExpression {
        if self.var_type != VariableType::Boolean {
            panic!("NOT operation is only defined for boolean variables");
        }
        SemanticExpression::logical(LogicalExpression::not(LogicalExpression::variable(self)))
    }
}

impl_var_binary_operation!(Add, add, addition);
impl_var_binary_operation!(Sub, sub, subtraction);
impl_var_binary_operation!(Mul, mul, multiplication);
impl_var_binary_operation!(Div, div, division);

impl_var_binary_operation!(BitAnd, bitand, and);
impl_var_binary_operation!(BitOr, bitor, or);

impl_var_unary_operation!(Neg, neg, negation);
impl_var_unary_operation!(Not, not, not);

impl_var_numeric_operation!(Add, add, add_expr, i32, i64, f32, f64, u32, u64);
impl_var_numeric_operation!(Sub, sub, sub_expr, i32, i64, f32, f64, u32, u64);
impl_var_numeric_operation!(Mul, mul, mul_expr, i32, i64, f32, f64, u32, u64);
impl_var_numeric_operation!(Div, div, div_expr, i32, i64, f32, f64, u32, u64);

impl_var_logic_operation!(BitAnd, bitand, and_expr, bool);
impl_var_logic_operation!(BitOr, bitor, or_expr, bool);

impl_var_expr_binary_operation!(Add, add, add_expr);
impl_var_expr_binary_operation!(Sub, sub, sub_expr);
impl_var_expr_binary_operation!(Mul, mul, mul_expr);
impl_var_expr_binary_operation!(Div, div, div_expr);

impl_var_expr_binary_operation!(BitAnd, bitand, and_expr);
impl_var_expr_binary_operation!(BitOr, bitor, or_expr);
