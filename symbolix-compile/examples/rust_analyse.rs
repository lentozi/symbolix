use symbolix_compile::symbolix_rust;

fn main() {
    symbolix_rust! {
        let y = var!("hello", i32);


        // z = var!("z", i32);

        let expr = expr!("hello + 2 * 10");

        let expr = expr + y;

        let equation = expr.equal_to(y);

        let result = equation.solve();

        (result, x)
    };
}
