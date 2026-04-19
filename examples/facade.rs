use exprion::{scope, Var};

fn main() {
    scope(|| {
        let x = Var::number("x");
        let y = Var::number("y");
        let z = Var::number("z");

        let equation = (&x + 2.0).eq_expr(6.0);
        let solved = equation
            .solve_unique()
            .expect("failed to solve single-variable equation");

        let expr = &z + &x * 2.0 + 1.0;
        let compiled = expr.jit_compile().expect("failed to JIT compile expression");
        let result = compiled
            .calculate_named(&[("z", 10.0), ("x", 3.0)])
            .expect("failed to execute JIT function");

        let relation = (&x + &y).gt(10.0);

        println!("equation: {} -> {}", equation.semantic(), solved.semantic());
        println!("expr: {}", expr.semantic());
        println!("parameters: {:?}", compiled.parameters());
        println!("result: {result}");
        println!("relation: {}", relation.semantic());
    });
}
