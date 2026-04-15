use symbolix_compile::symbolix;

fn main() {
    let code = symbolix! {
        let y = var!("z", f64);

        let expr = if y >= 10 {
            expr!("z - 2 * 10")
        } else {
            expr!("z * 2")
        };

        let expr = expr + y;

        let equation = expr.equal_to(y);

        let result = solve!(equation);

        (result, y)
        // result
    };

    println!("{}", code.calculate(2.0).0);
}
