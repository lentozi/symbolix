use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-env-changed=LLVM_PREFIX");
    println!("cargo:rerun-if-env-changed=LLVM_CONFIG_PATH");

    let (libdir, bin_dir) = locate_llvm();

    println!("cargo:rustc-link-search=native={}", libdir.display());
    println!("cargo:rustc-link-lib=dylib=LLVM-C");

    if cfg!(target_os = "windows") {
        let dll_dir = bin_dir.unwrap_or_else(|| libdir.clone());
        println!("cargo:rustc-env=PATH={};{}", dll_dir.display(), env::var("PATH").unwrap_or_default());
    }
}

fn locate_llvm() -> (PathBuf, Option<PathBuf>) {
    if let Ok(prefix) = env::var("LLVM_PREFIX") {
        let prefix = PathBuf::from(prefix);
        return (prefix.join("lib"), Some(prefix.join("bin")));
    }

    let llvm_config = env::var("LLVM_CONFIG_PATH").unwrap_or_else(|_| "llvm-config".to_string());
    let libdir = run_llvm_config(&llvm_config, "--libdir");
    let bindir = run_llvm_config(&llvm_config, "--bindir");
    (PathBuf::from(libdir), Some(PathBuf::from(bindir)))
}

fn run_llvm_config(llvm_config: &str, arg: &str) -> String {
    let output = Command::new(llvm_config)
        .arg(arg)
        .output()
        .unwrap_or_else(|err| panic!("failed to execute `{llvm_config} {arg}`: {err}"));

    if !output.status.success() {
        panic!(
            "`{llvm_config} {arg}` failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .to_string()
}
