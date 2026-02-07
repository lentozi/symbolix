use symbolix_core::{
    context::compile::CompileContext,
    lexer::{symbol::Precedence, Lexer},
    parser::{expression::Expression, pratt_parsing},
    semantic::semantic_without_ctx,
};

fn main() {
    let mut ctx = CompileContext::new();
    let input = "-x + x + 123 + 45.67 * ((89 - 0.1) ^ x) ^ x + 0";
    let mut lexer: Lexer = Lexer::new(input);
    let expression: Expression = pratt_parsing(&mut lexer, Precedence::Lowest);
    let semantic = semantic_without_ctx(&expression, true, &mut ctx);
    println!("{}", semantic);
}
