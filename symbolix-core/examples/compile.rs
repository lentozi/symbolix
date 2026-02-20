use symbolix_core::{
    lexer::{symbol::Precedence, Lexer},
    new_compile_context,
    optimizer::optimize,
    parser::pratt_parsing,
    semantic::semantic_without_ctx,
};

fn main() {
    new_compile_context! {
        let expr_str = "-x + y + 123 + 45.67 * ((89 - 0.1) ^ x) ^ x + 0";

        let mut lexer = Lexer::new(&expr_str);
        let expression = pratt_parsing(&mut lexer, Precedence::Lowest);
        let mut semantic_expression = semantic_without_ctx(&expression, true);
        optimize(&mut semantic_expression);
        println!("{}", semantic_expression);

    }
}
