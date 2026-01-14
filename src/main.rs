use crate::lexer::symbol::Precedence;
use crate::lexer::Lexer;
use crate::parser::expression::Expression;
use crate::semantic::ast_to_semantic;
use crate::semantic::variable::VariableType;

mod lexer;
mod parser;
mod semantic;
mod error;
mod macros;

fn main() {
    context! {
        let _a = var!("a", VariableType::Integer, None);
        let _b = var!("b", VariableType::Integer, None);
        let _c = var!("c", VariableType::Integer, None);
        let _d = var!("d", VariableType::Integer, None);
        let _e = var!("e", VariableType::Integer, None);

        // let input = "-x + 123 + 45.67 * (89 - 0.1) ^ x";
        // let input = "(x > 100 ? x * (2 + 3) : x) / 2";
        // let input = "1 * (2 + 3) * 4";
        let input = "a + b * c - d / e";
        let mut lexer: Lexer = Lexer::new(input);
        let expression: Expression = parser::pratt_parsing(&mut lexer, Precedence::Lowest);
        println!("{}", expression);
        let semantic_expression = ast_to_semantic(&expression);
        println!("{}", semantic_expression);
    }
}
