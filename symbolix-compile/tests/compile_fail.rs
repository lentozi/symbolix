use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn make_temp_crate_dir() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("symbolix_compile_fail_{unique}"))
}

fn write_temp_crate(main_rs: &str) -> PathBuf {
    let temp_dir = make_temp_crate_dir();
    let crate_dir = temp_dir.join("case");
    let src_dir = crate_dir.join("src");
    fs::create_dir_all(&src_dir).expect("failed to create temp crate");

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest_dir = manifest_dir
        .canonicalize()
        .expect("failed to canonicalize manifest dir");
    let manifest_dir = manifest_dir.to_string_lossy().replace('\\', "\\\\");

    fs::write(
        crate_dir.join("Cargo.toml"),
        format!(
            r#"[package]
name = "symbolix-compile-fail-case"
version = "0.1.0"
edition = "2021"

[dependencies]
symbolix-compile = {{ path = "{manifest_dir}" }}
"#,
        ),
    )
    .expect("failed to write Cargo.toml");

    fs::write(src_dir.join("main.rs"), main_rs).expect("failed to write main.rs");
    temp_dir
}

fn cargo_check_stderr(temp_dir: &PathBuf) -> String {
    let crate_dir = temp_dir.join("case");
    let output = Command::new("cargo")
        .arg("check")
        .current_dir(&crate_dir)
        .output()
        .expect("failed to run cargo check");

    assert!(
        !output.status.success(),
        "expected compile failure, got success:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn symbolix_reports_errors_inside_macro_body() {
    let temp_dir = write_temp_crate(
        r#"use symbolix_compile::symbolix;

fn main() {
    let _ = symbolix! {
        solve!(missing)
    };
}
"#,
    );
    let stderr = cargo_check_stderr(&temp_dir);
    assert!(
        stderr.contains("undefined binding `missing` in symbolix! block"),
        "missing error message in stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("solve!(missing)"),
        "missing source line in stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("|                ^^^^^^^"),
        "error was not highlighted on the macro body token:\n{stderr}"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn symbolix_reports_missing_else_branch() {
    let temp_dir = write_temp_crate(
        r#"use symbolix_compile::symbolix;

fn main() {
    let _ = symbolix! {
        let x = var!("x", f64);
        let zero = expr!("0");
        if x.greater_than(zero) {
            x
        }
    };
}
"#,
    );
    let stderr = cargo_check_stderr(&temp_dir);
    assert!(stderr.contains("requires an else branch"), "stderr:\n{stderr}");
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn symbolix_reports_unsupported_equation_method_call() {
    let temp_dir = write_temp_crate(
        r#"use symbolix_compile::symbolix;

fn main() {
    let _ = symbolix! {
        let x = var!("x", f64);
        let equation = x.equal_to(x);
        equation.solve()
    };
}
"#,
    );
    let stderr = cargo_check_stderr(&temp_dir);
    assert!(stderr.contains("use solve!(equation)"), "stderr:\n{stderr}");
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn symbolix_reports_non_identifier_let_pattern() {
    let temp_dir = write_temp_crate(
        r#"use symbolix_compile::symbolix;

fn main() {
    let _ = symbolix! {
        let (x, y) = (expr!("1"), expr!("2"));
        x
    };
}
"#,
    );
    let stderr = cargo_check_stderr(&temp_dir);
    assert!(stderr.contains("only supports identifier bindings"), "stderr:\n{stderr}");
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn symbolix_reports_invalid_var_type() {
    let temp_dir = write_temp_crate(
        r#"use symbolix_compile::symbolix;

fn main() {
    let _ = symbolix! {
        let x = var!("x", String);
        x
    };
}
"#,
    );
    let stderr = cargo_check_stderr(&temp_dir);
    assert!(stderr.contains("var! only supports"), "stderr:\n{stderr}");
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn symbolix_reports_method_receiver_must_be_named_binding() {
    let temp_dir = write_temp_crate(
        r#"use symbolix_compile::symbolix;

fn main() {
    let _ = symbolix! {
        (expr!("1") + expr!("2")).greater_than(expr!("0"))
    };
}
"#,
    );
    let stderr = cargo_check_stderr(&temp_dir);
    assert!(stderr.contains("method receiver in symbolix! must be a named binding"), "stderr:\n{stderr}");
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn symbolix_reports_solution_sets_cannot_participate_in_arithmetic() {
    let temp_dir = write_temp_crate(
        r#"use symbolix_compile::symbolix;

fn main() {
    let _ = symbolix! {
        let x = var!("x", f64);
        let lhs = expr!("x ^ 2 - 1");
        let rhs = expr!("0");
        let equation = lhs.equal_to(rhs);
        let solved = solve!(equation, x);
        solved + x
    };
}
"#,
    );
    let stderr = cargo_check_stderr(&temp_dir);
    assert!(stderr.contains("solution sets cannot participate"), "stderr:\n{stderr}");
    let _ = fs::remove_dir_all(&temp_dir);
}
