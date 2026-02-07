use proc_macro::TokenStream;
use symbolix_core::{
    lexer::{symbol::Precedence, Lexer},
    parser::pratt_parsing,
    semantic::ast_to_semantic,
};
use syn::{parse_macro_input, LitStr};

use crate::codegen::codegen_semantic;

mod codegen;

#[proc_macro]
pub fn compile(input: TokenStream) -> TokenStream {
    let input_lit = parse_macro_input!(input as LitStr);
    let expr_str = input_lit.value();

    let mut lexer = Lexer::new(&expr_str);
    let expression = pratt_parsing(&mut lexer, Precedence::Lowest);
    let semantic_expression = ast_to_semantic(&expression);

    let code = codegen_semantic(&semantic_expression);

    code.into()
}
