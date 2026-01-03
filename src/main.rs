use crate::expr::Expression;
use crate::lexer::Lexer;
use crate::lexer::symbol::Precedence;

mod lexer;
mod expr;

fn main() {
    // let input = "-x + 123 + 45.67 * (89 - 0.1) ^ x";
    let input = "(x > 100 ? x * (2 + 3) : x) / 2";
    // let input = "1 * (2 + 3) * 4";
    let mut lexer: Lexer = Lexer::new(input);
    // let expression: Expression = expr::parse_expression2(&mut lexer);
    let expression: Expression = expr::pratt_parsing(&mut lexer, Precedence::Lowest);
    println!("{}", expression);
}
