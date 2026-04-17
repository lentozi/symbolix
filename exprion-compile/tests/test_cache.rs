use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn make_temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("exprion_compile_{name}_{unique}"))
}

fn write_temp_formula_crate(body: &str) -> (PathBuf, PathBuf) {
    let temp_dir = make_temp_dir("cache");
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
name = "exprion-compile-cache-case"
version = "0.1.0"
edition = "2021"

[dependencies]
exprion-compile = {{ path = "{manifest_dir}" }}
"#,
        ),
    )
    .expect("failed to write Cargo.toml");

    fs::write(src_dir.join("main.rs"), body).expect("failed to write main.rs");
    (temp_dir, crate_dir)
}

#[test]
fn formula_uses_project_specific_cache_directory_when_configured() {
    let (temp_dir, crate_dir) = write_temp_formula_crate(
        r#"use exprion_compile::formula;

fn main() {
    let compiled = formula!("a + b * 2");
    let _ = compiled.calculate(1.0, 2.0);
}
"#,
    );
    let cache_dir = temp_dir.join("macro-cache");

    let output = Command::new("cargo")
        .arg("check")
        .current_dir(&crate_dir)
        .env("CARGO_NET_OFFLINE", "true")
        .env("EXPRION_CACHE_DIR", &cache_dir)
        .env("EXPRION_SILENCE_CACHE_WARNING", "1")
        .output()
        .expect("failed to run cargo check");

    assert!(
        output.status.success(),
        "cargo check failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let entries = fs::read_dir(&cache_dir)
        .expect("cache dir was not created")
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to read cache dir");
    assert!(!entries.is_empty(), "expected cache files to be created");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn formula_respects_disabled_cache_environment_variable() {
    let (temp_dir, crate_dir) = write_temp_formula_crate(
        r#"use exprion_compile::formula;

fn main() {
    let compiled = formula!("x + 1");
    let _ = compiled.calculate(1.0);
}
"#,
    );
    let cache_dir = temp_dir.join("disabled-cache");

    let output = Command::new("cargo")
        .arg("check")
        .current_dir(&crate_dir)
        .env("CARGO_NET_OFFLINE", "true")
        .env("EXPRION_CACHE_DIR", &cache_dir)
        .env("EXPRION_DISABLE_CACHE", "1")
        .env("EXPRION_SILENCE_CACHE_WARNING", "1")
        .output()
        .expect("failed to run cargo check");

    assert!(
        output.status.success(),
        "cargo check failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !cache_dir.exists() || fs::read_dir(&cache_dir).map(|mut it| it.next().is_none()).unwrap_or(true),
        "cache dir should be empty when cache is disabled"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}
