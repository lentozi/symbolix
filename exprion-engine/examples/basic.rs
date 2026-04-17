use exprion_core::{
    lexer::Lexer,
    new_compile_context,
    optimizer::optimize,
    parser::Parser,
    semantic::Analyzer,
};
use exprion_engine::jit_compile_numeric;

fn main() {
    let semantic = new_compile_context! {
        let mut lexer = Lexer::new("z + x * 2 + 1");
        let parsed = Parser::pratt(&mut lexer);
        let mut analyzer = Analyzer::new();
        let mut semantic = analyzer.analyze_with_ctx(&parsed);
        optimize(&mut semantic);
        semantic
    };
    let compiled = jit_compile_numeric(semantic).expect("failed to JIT compile semantic IR");
    let result = compiled
        .calculate_named(&[("z", 10.0), ("x", 3.0)])
        .expect("failed to execute JIT function");

    println!("parameters: {:?}", compiled.parameters());
    println!("result: {result}");
}
