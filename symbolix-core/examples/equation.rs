use symbolix_core::{
    equation::Equation, lexer::Lexer, new_compile_context, optimizer::optimize, parser::Parser,
    semantic::Analyzer,
};

fn main() {
    new_compile_context! {
        let expr_str = "(z > 10 ? 2 * z - 20 : 3 * z ^ 2) == z";

        let mut lexer = Lexer::new(&expr_str);
        let expression = Parser::pratt(&mut lexer);
        let mut semantic_expression = Analyzer::new().analyze_with_ctx(&expression);
        optimize(&mut semantic_expression);
        println!("{}", semantic_expression);

        let equation = Equation::infer(semantic_expression).unwrap();
        let result = equation.solve().unwrap();
        println!("{}", result);
    }
}
