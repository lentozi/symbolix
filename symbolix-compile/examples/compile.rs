use symbolix_compile::formula;

fn main() {
    let code = formula!("-x + y + 123 + 45.67 * ((89 - 0.1) ^ x) ^ x + 0");
    println!("{}", code.calculate(1.0, 100.0));

    let code = formula!("x + 1");
    println!("{}", code.calculate(5.0));

    let code = formula!("x == 50 ? 4 : 5");
    println!("{}", code.calculate(50.0))
}

