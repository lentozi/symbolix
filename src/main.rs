use crate::expr::Expression;
use crate::lexer::Lexer;

mod lexer;
mod expr;

fn main() {
    let input = "x + 123 + 45.67 * (89 - 0.1) ^ x";
    let lexer: Lexer = Lexer::new(input);
    let expression: Expression = expr::syntax_analyzer(lexer);
    println!("{:?}", expression);
}
