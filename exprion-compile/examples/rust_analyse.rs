use exprion_compile::exprion;

fn main() {
    let code = exprion! {
        let cond = var!("cond", bool);
        let y = var!("z", f64);

        let expr = if cond {
            expr!("z - 2 * 10")
        } else {
            expr!("z * 2")
        };

        let expr = expr + y;

        let equation = expr.equal_to(y);

        let result = solve!(equation, y);

        (result, y)
    };

    let (result, y) = code.calculate(true, 2.0);
    println!("y = {y}");
    println!("{result}");
}
