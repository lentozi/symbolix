// #[macro_export]
// macro_rules! impl_binary_operation {
//     ($trait:ident, $method:ident, $variant:ident) => {
//         impl $trait for $crate::semantic::semantic_ir::SemanticExpression {
//             type Output = Self;
//             fn $method(self, rhs: Self) -> Self::Output {
//                 $crate::semantic::semantic_ir::SemanticExpression::$variant(self, rhs)
//             }
//         }
//
//         impl $trait<&$crate::semantic::semantic_ir::SemanticExpression> for $crate::semantic::semantic_ir::SemanticExpression {
//             type Output = Self;
//             fn $method(self, rhs: &$crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
//                 $crate::semantic::semantic_ir::SemanticExpression::$variant(self, rhs.clone())
//             }
//         }
//
//         impl $trait<$crate::semantic::semantic_ir::SemanticExpression> for &$crate::semantic::semantic_ir::SemanticExpression {
//             type Output = $crate::semantic::semantic_ir::SemanticExpression;
//             fn $method(self, rhs: $crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
//                 $crate::semantic::semantic_ir::SemanticExpression::$variant(self.clone()), Box::new(rhs))
//             }
//         }
//
//         impl $trait<&$crate::semantic::semantic_ir::SemanticExpression> for $crate::semantic::semantic_ir::SemanticExpression {
//             type Output = $crate::semantic::semantic_ir::SemanticExpression;
//             fn $method(self, rhs: &$crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
//                 $crate::semantic::semantic_ir::SemanticExpression::$variant(Box::new(self.clone()), Box::new(rhs.clone()))
//             }
//         }
//     }
// }
//
// #[macro_export]
// macro_rules! impl_numeric_operation {
//     ($trait:ident, $method:ident, $variant:ident, $($num_type:ty),+) => {
//         $(
//             impl $trait<$num_type> for $crate::semantic::semantic_ir::SemanticExpression {
//                 type Output = Self;
//                 fn $method(self, rhs: $num_type) -> Self::Output {
//                     $crate::semantic::semantic_ir::SemanticExpression::Numeric(
//                         $crate::semantic::semantic_ir::NumericExpression::$variant(
//                             Box::new(self),
//                             Box::new($crate::semantic::semantic_ir::NumericExpression::Constant(rhs.into()))
//                         )
//                     )
//                 }
//             }
//
//             impl $trait<$num_type> for &$crate::semantic::semantic_ir::SemanticExpression {
//                 type Output = $crate::semantic::semantic_ir::SemanticExpression;
//                 fn $method(self, rhs: $num_type) -> Self::Output {
//                     $crate::semantic::semantic_ir::SemanticExpression::Numeric(
//                         $crate::semantic::semantic_ir::NumericExpression::$variant(
//                             Box::new(self.clone()),
//                             Box::new($crate::semantic::semantic_ir::NumericExpression::Constant(rhs.into()))
//                         )
//                     )
//                 }
//             }
//
//             impl $trait<$crate::semantic::semantic_ir::SemanticExpression> for $num_type {
//                 type Output = $crate::semantic::semantic_ir::SemanticExpression;
//                 fn $method(self, rhs: $crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
//                     $crate::semantic::semantic_ir::SemanticExpression::Numeric(
//                         $crate::semantic::semantic_ir::NumericExpression::$variant(
//                             Box::new($crate::semantic::semantic_ir::NumericExpression::Constant(self.into())),
//                             Box::new(rhs)
//                         )
//                     )
//                 }
//             }
//
//             impl $trait<&$crate::semantic::semantic_ir::SemanticExpression> for $num_type {
//                 type Output = $crate::semantic::semantic_ir::SemanticExpression;
//                 fn $method(self, rhs: &$crate::semantic::semantic_ir::SemanticExpression) -> Self::Output {
//                     $crate::semantic::semantic_ir::SemanticExpression::Numeric(
//                         $crate::semantic::semantic_ir::NumericExpression::$variant(
//                             Box::new($crate::semantic::semantic_ir::NumericExpression::Constant(self.into())),
//                             Box::new(rhs.clone())
//                         )
//                     )
//                 }
//             }
//         )+
//     };
// }