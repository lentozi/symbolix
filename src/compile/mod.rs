use crate::{
    context::Context, error::Error, lexer::constant::Constant,
    semantic::semantic_ir::SemanticExpression,
};

pub mod static_compile;

pub trait EvalFn {
    fn eval(&self) -> Result<Constant, Error>;
}

// use proc_macro2::TokenStream;
// use quote::{format_ident, quote};
// // 假设这是你项目中的结构，在宏 crate 中你需要相应地引入或重新定义
// // 实际工程中，通常使用 syn::Expr 来处理输入，或者将 AST 定义在 core crate 中
// use crate::lexer::constant::{Constant, Number};
// use crate::semantic::bucket::numeric::NumericBucket;
// use crate::semantic::semantic_ir::numeric::NumericExpression;

// pub fn codegen_numeric(expr: &NumericExpression) -> TokenStream {
//     match expr {
//         // 1. 常量：直接转换为 f64 字面量
//         NumericExpression::Constant(n) => {
//             let val = n.to_float(); // 假设你只需要 f64 运算
//             quote! { #val }
//         }

//         // 2. 变量：转换为对应的 Rust 变量标识符
//         NumericExpression::Variable(v) => {
//             let name = format_ident!("{}", v.name);
//             quote! { #name }
//         }

//         // 3. 负号：递归生成 -expr
//         NumericExpression::Negation(inner) => {
//             let inner_code = codegen_numeric(inner);
//             quote! { -#inner_code }
//         }

//         // 4. 加法：NumericBucket 是一个列表，需要把它们全部加起来
//         NumericExpression::Addition(bucket) => {
//             let mut terms = Vec::new();

//             // 处理常量部分
//             for c in &bucket.constants {
//                 let val = c.to_float();
//                 terms.push(quote! { #val });
//             }

//             // 处理变量部分
//             for v in &bucket.variables {
//                 let name = format_ident!("{}", v.name);
//                 terms.push(quote! { #name });
//             }

//             // 处理嵌套表达式部分
//             for e in &bucket.expressions {
//                 terms.push(codegen_numeric(e));
//             }

//             if terms.is_empty() {
//                 quote! { 0.0 }
//             } else {
//                 // 使用 + 连接所有项: term1 + term2 + term3...
//                 quote! { #(#terms)+* }
//             }
//         }

//         // 5. 乘法：逻辑同加法，用 * 连接
//         NumericExpression::Multiplication(bucket) => {
//             let mut terms = Vec::new();

//             // 处理分母/除法逻辑可能需要特别注意，
//             // 这里假设 bucket 里都是乘数
//             for c in &bucket.constants {
//                 let val = c.to_float();
//                 terms.push(quote! { #val });
//             }

//             for v in &bucket.variables {
//                 let name = format_ident!("{}", v.name);
//                 terms.push(quote! { #name });
//             }

//             for e in &bucket.expressions {
//                 terms.push(codegen_numeric(e));
//             }

//             if terms.is_empty() {
//                 quote! { 1.0 }
//             } else {
//                 quote! { #(#terms)* }
//             }
//         }

//         // 6. 幂运算
//         NumericExpression::Power { base, exponent } => {
//             let b = codegen_numeric(base);
//             let e = codegen_numeric(exponent);
//             // 生成 Rust 的 powf 调用
//             quote! { f64::powf(#b, #e) }
//         }

//         // 7. 分段函数 (Piecewise) - 这是一个难点
//         // 需要生成 if-else 链
//         NumericExpression::Piecewise { cases, otherwise } => {
//             // 示例逻辑，需要完善 LogicalExpression 的 codegen
//             let else_block = if let Some(other) = otherwise {
//                 codegen_numeric(other)
//             } else {
//                 quote! { f64::NAN } // 或者 panic
//             };

//             // 这里简化处理，实际上你需要 codegen_logical(cond)
//             let mut stream = else_block;

//             // 从后往前包裹 if else，或者使用 match
//             for (cond, val) in cases.iter().rev() {
//                 let val_code = codegen_numeric(val);
//                 // let cond_code = codegen_logical(cond);
//                 // 暂时用 true 占位
//                 let cond_code = quote! { true };

//                 stream = quote! {
//                     if #cond_code {
//                         #val_code
//                     } else {
//                         #stream
//                     }
//                 };
//             }
//             stream
//         }

//         // 处理 Division (虽然 NumericExpression 定义里好像没直接看到 Division，可能是乘法的一部分？)
//         // 查阅你的代码后发现 Division 在 impl NumericExpression 里有方法，但在 Enum 里没有？
//         // 再次检查 Enum 定义：只有 Constant, Variable, Negation, Addition, Multiplication, Power, Piecewise
//         // 看来除法在你的 IR 中被表示为 乘以倒数 (Multiplication) 或者 Power(x, -1)。
//         _ => quote! { compile_error!("Unsupported expression type") },
//     }
// }
