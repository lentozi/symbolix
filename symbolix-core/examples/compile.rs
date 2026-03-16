use symbolix_core::{
    lexer::Lexer,
    parser::Parser,
    semantic::Analyzer,
    new_compile_context,
    optimizer::optimize,
};

fn main() {
    new_compile_context! {
        let expr_str = "-x + y + 123 + 45.67 * ((89 - 0.1) ^ x) ^ x + 0";

        let mut lexer = Lexer::new(&expr_str);
        let expression = Parser::pratt(&mut lexer);
        let mut semantic_expression = Analyzer::new().analyze_with_ctx(&expression);
        optimize(&mut semantic_expression);
        println!("{}", semantic_expression);

    }
}
