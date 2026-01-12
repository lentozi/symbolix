use crate::parser::expression::Expression;
use crate::lexer::Lexer;
use crate::lexer::symbol::Precedence;
use crate::semantic::ast_to_semantic;
use crate::semantic::variable::{Variable, VariableType};

mod lexer;
mod parser;
mod semantic;
mod error;

fn main() {
    context! {
        let _a = Variable::new("a", VariableType::Integer);
        let _b = Variable::new("b", VariableType::Integer);
        let _c = Variable::new("c", VariableType::Integer);
        let _d = Variable::new("d", VariableType::Integer);
        let _e = Variable::new("e", VariableType::Integer);

        // let input = "-x + 123 + 45.67 * (89 - 0.1) ^ x";
        // let input = "(x > 100 ? x * (2 + 3) : x) / 2";
        // let input = "1 * (2 + 3) * 4";
        let input = "a + b * c - d / e";
        let mut lexer: Lexer = Lexer::new(input);
        let expression: Expression = parser::pratt_parsing(&mut lexer, Precedence::Lowest);
        println!("{}", expression);
        let semantic_expression = ast_to_semantic(&expression);
        println!("{:?}", semantic_expression);
    }
}
