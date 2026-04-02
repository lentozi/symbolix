use symbolix_compile::symbolix_rust;

fn main() {
    let code = symbolix_rust! {
        let y = var!("hello", f64);


        // z = var!("z", i32);

        let expr = expr!("hello + 2 * 10");

        let expr = expr + y;

        let equation = expr.equal_to(y);

        let result = equation.solve();

        (result, y)
    };

    println!("{}", code.calculate(10.0).0);
}
