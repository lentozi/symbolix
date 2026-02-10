use symbolix_compile::compile;

fn main() {
    let code = compile!("-x + y + 123 + 45.67 * ((89 - 0.1) ^ x) ^ x + 0");
    println!("{}", code.calculate(1.0, 100.0));

    let code = compile!("x + y");
    println!("{}", code.calculate(1.0, 100.0));
}
