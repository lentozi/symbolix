use symbolix_core::{
    equation::Equation, lexer::Lexer, new_compile_context, optimizer::optimize, parser::Parser,
    semantic::Analyzer,
};

fn main() {
    new_compile_context! {
        let expr_str = "3 * (x - 2) / 4 + 2 * (5 * x - (3 * x - 1)) / 3 == (1 - x) / 6 + 2";

        let mut lexer = Lexer::new(&expr_str);
        let expression = Parser::pratt(&mut lexer);
        let mut semantic_expression = Analyzer::new().analyze_with_ctx(&expression);
        optimize(&mut semantic_expression);
        println!("{}", semantic_expression);

        let equation = Equation::new(semantic_expression);
        let result = equation.solve().unwrap();
        println!("{}", result);
    }
}
