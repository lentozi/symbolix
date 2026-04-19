use exprion_api::{scope, Var};

fn main() {
    scope(|| {
        let x = Var::number("x");
        let y = Var::number("y");

        let inferred = (&x + 2.0).eq_expr(6.0);
        let inferred_solution = inferred
            .solve_unique()
            .expect("failed to solve single-variable equation");

        let explicit = (&x + &y).eq_expr(10.0);
        let explicit_solution = explicit
            .solve_for(&x)
            .expect("failed to solve equation for x")
            .into_expr()
            .expect("solution set is not representable as a single expression");

        println!("{} -> {}", inferred.semantic(), inferred_solution.semantic());
        println!("{} -> {}", explicit.semantic(), explicit_solution.semantic());
    });
}
