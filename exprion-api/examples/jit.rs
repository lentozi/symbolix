use exprion_api::{scope, Var};

fn main() {
    scope(|| {
        let x = Var::number("x");
        let z = Var::number("z");
        let expr = &z + &x * 2.0 + 1.0;

        let compiled = expr
            .jit_compile()
            .expect("failed to JIT compile API expression");
        let result = compiled
            .calculate_named(&[("z", 10.0), ("x", 3.0)])
            .expect("failed to execute JIT function");

        println!("expr: {}", expr.semantic());
        println!("parameters: {:?}", compiled.parameters());
        println!("result: {result}");
    });
}
