#[macro_export]
macro_rules! impl_var_binary_operation {
        ($trait:ident, $method:ident, $variant:ident) => {
        impl $trait for $crate::semantic::variable::Variable {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self, rhs: $crate::semantic::variable::Variable) -> Self::Output {
                $crate::semantic::variable::Variable::$variant(self, rhs)
            }
        }

        impl $trait<&$crate::semantic::variable::Variable> for $crate::semantic::variable::Variable {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self, rhs: &$crate::semantic::variable::Variable) -> Self::Output {
                $crate::semantic::variable::Variable::$variant(self, rhs.clone())
            }
        }

        impl $trait<$crate::semantic::variable::Variable> for &$crate::semantic::variable::Variable {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self, rhs: $crate::semantic::variable::Variable) -> Self::Output {
                $crate::semantic::variable::Variable::$variant(self.clone(), rhs)
            }
        }

        impl $trait<&$crate::semantic::variable::Variable> for &$crate::semantic::variable::Variable {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self, rhs: &$crate::semantic::variable::Variable) -> Self::Output {
                $crate::semantic::variable::Variable::$variant(self.clone(), rhs.clone())
            }
        }
    }
}

#[macro_export]
macro_rules! impl_var_unary_operation {
    ($trait:ident, $method:ident, $variant:ident) => {
        impl $trait for $crate::semantic::variable::Variable {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self) -> Self::Output {
                $crate::semantic::variable::Variable::$variant(self)
            }
        }

        impl $trait for &$crate::semantic::variable::Variable {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self) -> Self::Output {
                $crate::semantic::variable::Variable::$variant(self.clone())
            }
        }
    };
}


#[macro_export]
macro_rules! impl_var_numeric_operation {
    ($trait:ident, $method:ident, $variant:ident, $($num_type:ty),+) => {
        $(
            impl $trait<$num_type> for $crate::semantic::variable::Variable {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: $num_type) -> Self::Output {
                    $crate::semantic::variable::Variable::$variant(self, $crate::semantic::semantic_ir::SemanticExpression::numeric($crate::semantic::semantic_ir::numeric::NumericExpression::compatible_constant(rhs as f64)))
                }
            }

            impl $trait<$num_type> for &$crate::semantic::variable::Variable {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: $num_type) -> Self::Output {
                    $crate::semantic::variable::Variable::$variant(self.clone(), $crate::semantic::semantic_ir::SemanticExpression::numeric($crate::semantic::semantic_ir::numeric::NumericExpression::compatible_constant(rhs as f64)))
                }
            }

            impl $trait<$crate::semantic::variable::Variable> for $num_type {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: $crate::semantic::variable::Variable) -> Self::Output {
                    $crate::semantic::variable::Variable::$variant(rhs, $crate::semantic::semantic_ir::SemanticExpression::numeric($crate::semantic::semantic_ir::numeric::NumericExpression::compatible_constant(self as f64)))
                }
            }

            impl $trait<&$crate::semantic::variable::Variable> for $num_type {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: &$crate::semantic::variable::Variable) -> Self::Output {
                    $crate::semantic::variable::Variable::$variant(rhs.clone(), $crate::semantic::semantic_ir::SemanticExpression::numeric($crate::semantic::semantic_ir::numeric::NumericExpression::compatible_constant(self as f64)))
                }
            }
        )+
    };
}

#[macro_export]
macro_rules! impl_var_logic_operation {
    ($trait:ident, $method:ident, $variant:ident, $($num_type:ty),+) => {
        $(
            impl $trait<$num_type> for $crate::semantic::variable::Variable {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: $num_type) -> Self::Output {
                    $crate::semantic::variable::Variable::$variant(self, $crate::semantic::semantic_ir::SemanticExpression::logical($crate::semantic::semantic_ir::logic::LogicalExpression::constant(rhs)))
                }
            }

            impl $trait<$num_type> for &$crate::semantic::variable::Variable {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: $num_type) -> Self::Output {
                    $crate::semantic::variable::Variable::$variant(self.clone(), $crate::semantic::semantic_ir::SemanticExpression::logical($crate::semantic::semantic_ir::logic::LogicalExpression::constant(rhs)))
                }
            }

            impl $trait<$crate::semantic::variable::Variable> for $num_type {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: $crate::semantic::variable::Variable) -> Self::Output {
                    $crate::semantic::variable::Variable::$variant(rhs, $crate::semantic::semantic_ir::SemanticExpression::logical($crate::semantic::semantic_ir::logic::LogicalExpression::constant(self)))
                }
            }

            impl $trait<&$crate::semantic::variable::Variable> for $num_type {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: &$crate::semantic::variable::Variable) -> Self::Output {
                    $crate::semantic::variable::Variable::$variant(rhs.clone(), $crate::semantic::semantic_ir::SemanticExpression::logical($crate::semantic::semantic_ir::logic::LogicalExpression::constant(self)))
                }
            }
        )+
    };
}

#[macro_export]
macro_rules! impl_var_expr_binary_operation {
    ($trait:ident, $method:ident, $variant:ident) => {
        impl $trait<$crate::semantic::semantic_ir::SemanticExpression> for $crate::semantic::variable::Variable {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self, rhs: $crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
                $crate::semantic::variable::Variable::$variant(self, rhs)
            }
        }

        impl $trait<&$crate::semantic::semantic_ir::SemanticExpression> for $crate::semantic::variable::Variable {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self, rhs: &$crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
                $crate::semantic::variable::Variable::$variant(self, rhs.clone())
            }
        }

        impl $trait<$crate::semantic::semantic_ir::SemanticExpression> for &$crate::semantic::variable::Variable {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self, rhs: SemanticExpression) -> Self::Output {
                $crate::semantic::variable::Variable::$variant(self.clone(), rhs)
            }
        }

        impl $trait<&$crate::semantic::semantic_ir::SemanticExpression> for &$crate::semantic::variable::Variable {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self, rhs: &$crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
                $crate::semantic::variable::Variable::$variant(self.clone(), rhs.clone())
            }
        }

        impl $trait<$crate::semantic::variable::Variable> for $crate::semantic::semantic_ir::SemanticExpression {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self, rhs: $crate::semantic::variable::Variable) -> Self::Output {
                $crate::semantic::variable::Variable::$variant(rhs, self)
            }
        }

        impl $trait<&$crate::semantic::variable::Variable> for $crate::semantic::semantic_ir::SemanticExpression {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self, rhs: &$crate::semantic::variable::Variable) -> Self::Output {
                $crate::semantic::variable::Variable::$variant(rhs.clone(), self)
            }
        }

        impl $trait<$crate::semantic::variable::Variable> for &$crate::semantic::semantic_ir::SemanticExpression {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self, rhs: $crate::semantic::variable::Variable) -> Self::Output {
                $crate::semantic::variable::Variable::$variant(rhs, self.clone())
            }
        }

        impl $trait<&$crate::semantic::variable::Variable> for &$crate::semantic::semantic_ir::SemanticExpression {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self, rhs: &$crate::semantic::variable::Variable) -> Self::Output {
                $crate::semantic::variable::Variable::$variant(rhs.clone(), self.clone())
            }
        }
    }
}