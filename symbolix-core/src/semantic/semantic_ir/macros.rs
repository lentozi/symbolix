#[macro_export]
macro_rules! impl_expr_binary_operation {
    ($trait:ident, $method:ident, $variant:ident) => {
        impl $trait for $crate::semantic::semantic_ir::SemanticExpression {
            type Output = Self;
            fn $method(self, rhs: Self) -> Self::Output {
                $crate::semantic::semantic_ir::SemanticExpression::$variant(&self, &rhs)
            }
        }

        impl $trait<&$crate::semantic::semantic_ir::SemanticExpression>
            for $crate::semantic::semantic_ir::SemanticExpression
        {
            type Output = Self;
            fn $method(
                self,
                rhs: &$crate::semantic::semantic_ir::SemanticExpression,
            ) -> Self::Output {
                $crate::semantic::semantic_ir::SemanticExpression::$variant(&self, rhs)
            }
        }

        impl $trait<$crate::semantic::semantic_ir::SemanticExpression>
            for &$crate::semantic::semantic_ir::SemanticExpression
        {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(
                self,
                rhs: $crate::semantic::semantic_ir::SemanticExpression,
            ) -> Self::Output {
                $crate::semantic::semantic_ir::SemanticExpression::$variant(self, &rhs)
            }
        }

        impl $trait<&$crate::semantic::semantic_ir::SemanticExpression>
            for &$crate::semantic::semantic_ir::SemanticExpression
        {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(
                self,
                rhs: &$crate::semantic::semantic_ir::SemanticExpression,
            ) -> Self::Output {
                $crate::semantic::semantic_ir::SemanticExpression::$variant(self, rhs)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_expr_unary_operation {
    ($trait:ident, $method:ident, $variant:ident) => {
        impl $trait for $crate::semantic::semantic_ir::SemanticExpression {
            type Output = Self;
            fn $method(self) -> Self::Output {
                $crate::semantic::semantic_ir::SemanticExpression::$variant(&self)
            }
        }

        impl $trait for &$crate::semantic::semantic_ir::SemanticExpression {
            type Output = $crate::semantic::semantic_ir::SemanticExpression;
            fn $method(self) -> Self::Output {
                $crate::semantic::semantic_ir::SemanticExpression::$variant(self)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_expr_numeric_operation {
    ($trait:ident, $method:ident, $variant:ident, $($num_type:ty),+) => {
        $(
            impl $trait<$num_type> for $crate::semantic::semantic_ir::SemanticExpression {
                type Output = Self;
                fn $method(self, rhs: $num_type) -> Self::Output {
                    $crate::semantic::semantic_ir::SemanticExpression::$variant(&self, &$crate::semantic::semantic_ir::SemanticExpression::numeric($crate::semantic::semantic_ir::numeric::NumericExpression::compatible_constant(rhs as f64)))
                }
            }

            impl $trait<$num_type> for &$crate::semantic::semantic_ir::SemanticExpression {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: $num_type) -> Self::Output {
                    $crate::semantic::semantic_ir::SemanticExpression::$variant(self, &$crate::semantic::semantic_ir::SemanticExpression::numeric($crate::semantic::semantic_ir::numeric::NumericExpression::compatible_constant(rhs as f64)))
                }
            }

            impl $trait<$crate::semantic::semantic_ir::SemanticExpression> for $num_type {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: $crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
                    $crate::semantic::semantic_ir::SemanticExpression::$variant(&$crate::semantic::semantic_ir::SemanticExpression::numeric($crate::semantic::semantic_ir::numeric::NumericExpression::compatible_constant(self as f64)), &rhs)
                }
            }

            impl $trait<&$crate::semantic::semantic_ir::SemanticExpression> for $num_type {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: &$crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
                    $crate::semantic::semantic_ir::SemanticExpression::$variant(&$crate::semantic::semantic_ir::SemanticExpression::numeric($crate::semantic::semantic_ir::numeric::NumericExpression::compatible_constant(self as f64)), rhs)
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
                    $crate::semantic::semantic_ir::SemanticExpression::$variant(&self, &$crate::semantic::semantic_ir::SemanticExpression::logical($crate::semantic::semantic_ir::logic::LogicalExpression::constant(rhs)))
                }
            }

            impl $trait<$num_type> for &$crate::semantic::semantic_ir::SemanticExpression {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: $num_type) -> Self::Output {
                    $crate::semantic::semantic_ir::SemanticExpression::$variant(self, &$crate::semantic::semantic_ir::SemanticExpression::logical($crate::semantic::semantic_ir::logic::LogicalExpression::constant(rhs)))
                }
            }

            impl $trait<$crate::semantic::semantic_ir::SemanticExpression> for $num_type {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: $crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
                    $crate::semantic::semantic_ir::SemanticExpression::$variant(&$crate::semantic::semantic_ir::SemanticExpression::logical($crate::semantic::semantic_ir::logic::LogicalExpression::constant(self)), &rhs)
                }
            }

            impl $trait<&$crate::semantic::semantic_ir::SemanticExpression> for $num_type {
                type Output = $crate::semantic::semantic_ir::SemanticExpression;
                fn $method(self, rhs: &$crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
                    $crate::semantic::semantic_ir::SemanticExpression::$variant(&$crate::semantic::semantic_ir::SemanticExpression::logical($crate::semantic::semantic_ir::logic::LogicalExpression::constant(self)), rhs)
                }
            }
        )+
    };
}

#[macro_export]
macro_rules! impl_numeric_ir_binary_operation {
    ($trait:ident, $method:ident, $variant:ident) => {
        impl $trait for $crate::semantic::semantic_ir::NumericExpression {
            type Output = Self;
            fn $method(self, rhs: Self) -> Self::Output {
                $crate::semantic::semantic_ir::NumericExpression::$variant(&self, &rhs)
            }
        }

        impl $trait<&$crate::semantic::semantic_ir::NumericExpression>
            for $crate::semantic::semantic_ir::NumericExpression
        {
            type Output = Self;
            fn $method(
                self,
                rhs: &$crate::semantic::semantic_ir::NumericExpression,
            ) -> Self::Output {
                $crate::semantic::semantic_ir::NumericExpression::$variant(&self, rhs)
            }
        }

        impl $trait<$crate::semantic::semantic_ir::NumericExpression>
            for &$crate::semantic::semantic_ir::NumericExpression
        {
            type Output = $crate::semantic::semantic_ir::NumericExpression;
            fn $method(
                self,
                rhs: $crate::semantic::semantic_ir::NumericExpression,
            ) -> Self::Output {
                $crate::semantic::semantic_ir::NumericExpression::$variant(self, &rhs)
            }
        }

        impl $trait<&$crate::semantic::semantic_ir::NumericExpression>
            for &$crate::semantic::semantic_ir::NumericExpression
        {
            type Output = $crate::semantic::semantic_ir::NumericExpression;
            fn $method(
                self,
                rhs: &$crate::semantic::semantic_ir::NumericExpression,
            ) -> Self::Output {
                $crate::semantic::semantic_ir::NumericExpression::$variant(self, rhs)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_numeric_ir_unary_operation {
    ($trait:ident, $method:ident, $variant:ident) => {
        impl $trait for $crate::semantic::semantic_ir::NumericExpression {
            type Output = Self;
            fn $method(self) -> Self::Output {
                $crate::semantic::semantic_ir::NumericExpression::$variant(&self)
            }
        }

        impl $trait for &$crate::semantic::semantic_ir::NumericExpression {
            type Output = $crate::semantic::semantic_ir::NumericExpression;
            fn $method(self) -> Self::Output {
                $crate::semantic::semantic_ir::NumericExpression::$variant(self)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_logic_ir_binary_operation {
    ($trait:ident, $method:ident, $variant:ident) => {
        impl $trait for $crate::semantic::semantic_ir::LogicalExpression {
            type Output = Self;
            fn $method(
                self,
                rhs: $crate::semantic::semantic_ir::LogicalExpression,
            ) -> Self::Output {
                $crate::semantic::semantic_ir::LogicalExpression::$variant(&self, &rhs)
            }
        }

        impl $trait<&$crate::semantic::semantic_ir::LogicalExpression>
            for $crate::semantic::semantic_ir::LogicalExpression
        {
            type Output = Self;
            fn $method(
                self,
                rhs: &$crate::semantic::semantic_ir::LogicalExpression,
            ) -> Self::Output {
                $crate::semantic::semantic_ir::LogicalExpression::$variant(&self, rhs)
            }
        }

        impl $trait<$crate::semantic::semantic_ir::LogicalExpression>
            for &$crate::semantic::semantic_ir::LogicalExpression
        {
            type Output = $crate::semantic::semantic_ir::LogicalExpression;
            fn $method(
                self,
                rhs: $crate::semantic::semantic_ir::LogicalExpression,
            ) -> Self::Output {
                $crate::semantic::semantic_ir::LogicalExpression::$variant(self, &rhs)
            }
        }

        impl $trait<&$crate::semantic::semantic_ir::LogicalExpression>
            for &$crate::semantic::semantic_ir::LogicalExpression
        {
            type Output = $crate::semantic::semantic_ir::LogicalExpression;
            fn $method(
                self,
                rhs: &$crate::semantic::semantic_ir::LogicalExpression,
            ) -> Self::Output {
                $crate::semantic::semantic_ir::LogicalExpression::$variant(self, rhs)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_logic_ir_unary_operation {
    ($trait:ident, $method:ident, $variant:ident) => {
        impl $trait for $crate::semantic::semantic_ir::LogicalExpression {
            type Output = Self;
            fn $method(self) -> Self::Output {
                $crate::semantic::semantic_ir::LogicalExpression::$variant(&self)
            }
        }

        impl $trait for &$crate::semantic::semantic_ir::LogicalExpression {
            type Output = $crate::semantic::semantic_ir::LogicalExpression;
            fn $method(self) -> Self::Output {
                $crate::semantic::semantic_ir::LogicalExpression::$variant(self)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_logic_ir_logic_operation {
    ($trait:ident, $method:ident, $variant:ident, $($num_type:ty),+) => {
        $(
            impl $trait<$num_type> for $crate::semantic::semantic_ir::LogicalExpression {
                type Output = Self;
                fn $method(self, rhs: $num_type) -> Self::Output {
                    $crate::semantic::semantic_ir::LogicalExpression::$variant(&self, &$crate::semantic::semantic_ir::logic::LogicalExpression::constant(rhs))
                }
            }

            impl $trait<$num_type> for &$crate::semantic::semantic_ir::LogicalExpression {
                type Output = $crate::semantic::semantic_ir::LogicalExpression;
                fn $method(self, rhs: $num_type) -> Self::Output {
                    $crate::semantic::semantic_ir::LogicalExpression::$variant(self, &$crate::semantic::semantic_ir::logic::LogicalExpression::constant(rhs))
                }
            }

            impl $trait<$crate::semantic::semantic_ir::LogicalExpression> for $num_type {
                type Output = $crate::semantic::semantic_ir::LogicalExpression;
                fn $method(self, rhs: $crate::semantic::semantic_ir::LogicalExpression) -> Self::Output {
                    $crate::semantic::semantic_ir::LogicalExpression::$variant(&$crate::semantic::semantic_ir::logic::LogicalExpression::constant(self), &rhs)
                }
            }

            impl $trait<&$crate::semantic::semantic_ir::LogicalExpression> for $num_type {
                type Output = $crate::semantic::semantic_ir::LogicalExpression;
                fn $method(self, rhs: &$crate::semantic::semantic_ir::LogicalExpression) -> Self::Output {
                    $crate::semantic::semantic_ir::LogicalExpression::$variant(&$crate::semantic::semantic_ir::logic::LogicalExpression::constant(self), rhs)
                }
            }
        )+
    };
}

#[macro_export]
macro_rules! impl_numeric_ir_numeric_operation {
    ($trait:ident, $method:ident, $variant:ident, $($num_type:ty),+) => {
        $(
            impl $trait<$num_type> for $crate::semantic::semantic_ir::NumericExpression {
                type Output = Self;
                fn $method(self, rhs: $num_type) -> Self::Output {
                    $crate::semantic::semantic_ir::NumericExpression::$variant(&self, &$crate::semantic::semantic_ir::numeric::NumericExpression::compatible_constant(rhs as f64))
                }
            }

            impl $trait<$num_type> for &$crate::semantic::semantic_ir::NumericExpression {
                type Output = $crate::semantic::semantic_ir::NumericExpression;
                fn $method(self, rhs: $num_type) -> Self::Output {
                    $crate::semantic::semantic_ir::NumericExpression::$variant(self, &$crate::semantic::semantic_ir::numeric::NumericExpression::compatible_constant(rhs as f64))
                }
            }

            impl $trait<$crate::semantic::semantic_ir::NumericExpression> for $num_type {
                type Output = $crate::semantic::semantic_ir::NumericExpression;
                fn $method(self, rhs: $crate::semantic::semantic_ir::NumericExpression) -> Self::Output {
                    $crate::semantic::semantic_ir::NumericExpression::$variant(&$crate::semantic::semantic_ir::numeric::NumericExpression::compatible_constant(self as f64), &rhs)
                }
            }

            impl $trait<&$crate::semantic::semantic_ir::NumericExpression> for $num_type {
                type Output = $crate::semantic::semantic_ir::NumericExpression;
                fn $method(self, rhs: &$crate::semantic::semantic_ir::NumericExpression) -> Self::Output {
                    $crate::semantic::semantic_ir::NumericExpression::$variant(&$crate::semantic::semantic_ir::numeric::NumericExpression::compatible_constant(self as f64), rhs)
                }
            }
        )+
    };
}
