use symbolix_compile::symbolix_rust;

fn main() {
    let code = symbolix_rust! {
        let y = var!("x", f64);


        // z = var!("z", i32);

        let expr = expr!("hello + 2");

        let expr = if y >= 10 {
            expr!("hello + 2 * 10")
        } else {
            expr!("hello * 2")
        };

        let expr = expr + y;

        // let equation = expr.equal_to(y);

        // let result = equation.solve();

        (expr, y)
    };

    println!("{}", code.calculate(10.0, 5.0).0);
}
