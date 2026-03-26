#[macro_export]
macro_rules! impl_number_binary_operation {
    ($trait:ident, $method:ident, $variant:ident) => {
        impl $trait for $crate::lexer::constant::Number {
            type Output = Self;
            fn $method(self, rhs: $crate::lexer::constant::Number) -> Self::Output {
                $crate::lexer::constant::Number::$variant(&self, &rhs)
            }
        }

        impl $trait<&$crate::lexer::constant::Number> for $crate::lexer::constant::Number {
            type Output = Self;
            fn $method(self, rhs: &$crate::lexer::constant::Number) -> Self::Output {
                $crate::lexer::constant::Number::$variant(&self, rhs)
            }
        }

        impl $trait<$crate::lexer::constant::Number> for &$crate::lexer::constant::Number {
            type Output = $crate::lexer::constant::Number;
            fn $method(self, rhs: $crate::lexer::constant::Number) -> Self::Output {
                $crate::lexer::constant::Number::$variant(self, &rhs)
            }
        }

        impl $trait<&$crate::lexer::constant::Number> for &$crate::lexer::constant::Number {
            type Output = $crate::lexer::constant::Number;
            fn $method(self, rhs: &$crate::lexer::constant::Number) -> Self::Output {
                $crate::lexer::constant::Number::$variant(self, rhs)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_number_unary_operation {
    ($trait:ident, $method:ident, $variant:ident) => {
        impl $trait for $crate::lexer::constant::Number {
            type Output = Self;
            fn $method(self) -> Self::Output {
                $crate::lexer::constant::Number::$variant(&self)
            }
        }

        impl $trait for &$crate::lexer::constant::Number {
            type Output = $crate::lexer::constant::Number;
            fn $method(self) -> Self::Output {
                $crate::lexer::constant::Number::$variant(self)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_constant_binary_operation {
    ($trait:ident, $method:ident, $variant:ident) => {
        impl $trait for $crate::lexer::constant::Constant {
            type Output = Self;
            fn $method(self, rhs: $crate::lexer::constant::Constant) -> Self::Output {
                $crate::lexer::constant::Constant::$variant(&self, &rhs)
            }
        }

        impl $trait<$crate::lexer::constant::Constant> for &$crate::lexer::constant::Constant {
            type Output = $crate::lexer::constant::Constant;
            fn $method(self, rhs: $crate::lexer::constant::Constant) -> Self::Output {
                $crate::lexer::constant::Constant::$variant(self, &rhs)
            }
        }

        impl $trait<&$crate::lexer::constant::Constant> for $crate::lexer::constant::Constant {
            type Output = $crate::lexer::constant::Constant;
            fn $method(self, rhs: &$crate::lexer::constant::Constant) -> Self::Output {
                $crate::lexer::constant::Constant::$variant(&self, rhs)
            }
        }

        impl $trait<&$crate::lexer::constant::Constant> for &$crate::lexer::constant::Constant {
            type Output = $crate::lexer::constant::Constant;
            fn $method(self, rhs: &$crate::lexer::constant::Constant) -> Self::Output {
                $crate::lexer::constant::Constant::$variant(self, rhs)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_constant_unary_operation {
    ($trait:ident, $method:ident, $variant:ident) => {
                impl $trait for $crate::lexer::constant::Constant {
            type Output = Self;
            fn $method(self) -> Self::Output {
                $crate::lexer::constant::Constant::$variant(&self)
            }
        }

        impl $trait for &$crate::lexer::constant::Constant {
            type Output = $crate::lexer::constant::Constant;
            fn $method(self) -> Self::Output {
                $crate::lexer::constant::Constant::$variant(self)
            }
        }
    };
}
