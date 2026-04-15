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

#[test]
fn symbolix_reports_errors_inside_macro_body() {
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

    fs::write(
        src_dir.join("main.rs"),
        r#"use symbolix_compile::symbolix;

fn main() {
    let _ = symbolix! {
        solve!(missing)
    };
}
"#,
    )
    .expect("failed to write main.rs");

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

    let stderr = String::from_utf8_lossy(&output.stderr);
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
