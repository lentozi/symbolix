#[macro_export]
macro_rules! impl_expr_binary_operation {
    ($trait:ident, $method:ident, $variant:ident) => {
        impl $trait for $crate::semantic::semantic_ir::SemanticExpression {
            type Output = Self;
            fn $method(self, rhs: Self) -> Self::Output {
                $crate::semantic::semantic_ir::SemanticExpression::$variant(self, rhs)
            }
        }

        impl $trait<&$crate::semantic::semantic_ir::SemanticExpression> for $crate::semantic::semantic_ir::SemanticExpression {
            type Output = Self;
            fn $method(self, rhs: &$crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
                $crate::semantic::semantic_ir::SemanticExpression::$variant(self, rhs.clone())
            }
        }

        impl $trait<$crate::semantic::semantic_ir::SemanticExpression> for &$crate::semantic::semantic_ir::SemanticExpression {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self, rhs: $crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
                $crate::semantic::semantic_ir::SemanticExpression::$variant(self.clone(), rhs)
            }
        }

        impl $trait<&$crate::semantic::semantic_ir::SemanticExpression> for &$crate::semantic::semantic_ir::SemanticExpression {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self, rhs: &$crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
                $crate::semantic::semantic_ir::SemanticExpression::$variant(self.clone(), rhs.clone())
            }
        }
    }
}

#[macro_export]
macro_rules! impl_expr_unary_operation {
    ($trait:ident, $method:ident, $variant:ident) => {
        impl $trait for $crate::semantic::semantic_ir::SemanticExpression {
            type Output = Self;
            fn $method(self) -> Self::Output {
                $crate::semantic::semantic_ir::SemanticExpression::$variant(self)
            }
        }

        impl $trait for &$crate::semantic::semantic_ir::SemanticExpression {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self) -> Self::Output {
                $crate::semantic::semantic_ir::SemanticExpression::$variant(self.clone())
            }
        }
    }
}

#[macro_export]
macro_rules! impl_expr_numeric_operation {
    ($trait:ident, $method:ident, $variant:ident, $($num_type:ty),+) => {
        $(
            impl $trait<$num_type> for $crate::semantic::semantic_ir::SemanticExpression {
                type Output = Self;
                fn $method(self, rhs: $num_type) -> Self::Output {
                    $crate::semantic::semantic_ir::SemanticExpression::$variant(self, $crate::semantic::semantic_ir::SemanticExpression::numeric($crate::semantic::semantic_ir::numeric::NumericExpression::compatible_constant(rhs as f64)))
                }
            }

            impl $trait<$num_type> for &$crate::semantic::semantic_ir::SemanticExpression {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: $num_type) -> Self::Output {
                    $crate::semantic::semantic_ir::SemanticExpression::$variant(self.clone(), $crate::semantic::semantic_ir::SemanticExpression::numeric($crate::semantic::semantic_ir::numeric::NumericExpression::compatible_constant(rhs as f64)))
                }
            }

            impl $trait<$crate::semantic::semantic_ir::SemanticExpression> for $num_type {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: $crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
                    $crate::semantic::semantic_ir::SemanticExpression::$variant($crate::semantic::semantic_ir::SemanticExpression::numeric($crate::semantic::semantic_ir::numeric::NumericExpression::compatible_constant(self as f64)), rhs)
                }
            }

            impl $trait<&$crate::semantic::semantic_ir::SemanticExpression> for $num_type {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: &$crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
                    $crate::semantic::semantic_ir::SemanticExpression::$variant($crate::semantic::semantic_ir::SemanticExpression::numeric($crate::semantic::semantic_ir::numeric::NumericExpression::compatible_constant(self as f64)), rhs.clone())
                }
            }
        )+
    };
}

#[macro_export]
macro_rules! impl_expr_logic_operation {
    ($trait:ident, $method:ident, $variant:ident, $($num_type:ty),+) => {
        $(
            impl $trait<$num_type> for $crate::semantic::semantic_ir::SemanticExpression {
                type Output = Self;
                fn $method(self, rhs: $num_type) -> Self::Output {
                    $crate::semantic::semantic_ir::SemanticExpression::$variant(self, $crate::semantic::semantic_ir::SemanticExpression::logical($crate::semantic::semantic_ir::logic::LogicalExpression::constant(rhs)))
                }
            }

            impl $trait<$num_type> for &$crate::semantic::semantic_ir::SemanticExpression {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: $num_type) -> Self::Output {
                    $crate::semantic::semantic_ir::SemanticExpression::$variant(self.clone(), $crate::semantic::semantic_ir::SemanticExpression::logical($crate::semantic::semantic_ir::logic::LogicalExpression::constant(rhs)))
                }
            }

            impl $trait<$crate::semantic::semantic_ir::SemanticExpression> for $num_type {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: $crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
                    $crate::semantic::semantic_ir::SemanticExpression::$variant($crate::semantic::semantic_ir::SemanticExpression::logical($crate::semantic::semantic_ir::logic::LogicalExpression::constant(self)), rhs)
                }
            }

            impl $trait<&$crate::semantic::semantic_ir::SemanticExpression> for $num_type {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: &$crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
                    $crate::semantic::semantic_ir::SemanticExpression::$variant($crate::semantic::semantic_ir::SemanticExpression::logical($crate::semantic::semantic_ir::logic::LogicalExpression::constant(self)), rhs.clone())
                }
            }
        )+
    };
}
