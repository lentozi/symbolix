use exprion_core::{lexer::Lexer, new_compile_context};

fn main() {
    new_compile_context! {
        let input = "123que";
        let mut lexer = Lexer::new(input);

        println!("{:?}", lexer.tokens());
    }
}
