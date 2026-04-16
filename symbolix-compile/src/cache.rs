use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::sync::Once;

use proc_macro::TokenStream;
const CACHE_SALT: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "|",
    include_str!("lib.rs"),
    "|",
    include_str!("expand.rs"),
    "|",
    include_str!("codegen.rs"),
    "|",
    include_str!("rust_expr.rs")
);
static FALLBACK_WARNING: Once = Once::new();

const CACHE_DIR_ENV: &str = "SYMBOLIX_CACHE_DIR";
const DISABLE_CACHE_ENV: &str = "SYMBOLIX_DISABLE_CACHE";
const SILENCE_WARNING_ENV: &str = "SYMBOLIX_SILENCE_CACHE_WARNING";

pub(crate) fn expand_with_cache<F>(
    macro_name: &str,
    normalized_input: &str,
    expand: F,
) -> syn::Result<TokenStream>
where
    F: FnOnce() -> syn::Result<TokenStream>,
{
    let cache_root = match cache_root() {
        CacheMode::Disabled => return expand(),
        CacheMode::Disk(path) => path,
    };

    let disk_key = stable_hash(&format!("{CACHE_SALT}|{normalized_input}"));
    if let Some(tokens) = read_disk_cache(&cache_root, macro_name, &disk_key)? {
        return Ok(tokens);
    }

    let tokens = expand()?;
    let rendered = tokens.to_string();
    write_disk_cache(&cache_root, macro_name, &disk_key, &rendered);
    Ok(tokens)
}

fn read_disk_cache(
    cache_root: &Path,
    macro_name: &str,
    disk_key: &str,
) -> syn::Result<Option<TokenStream>> {
    let cache_file = cache_file_path(cache_root, macro_name, disk_key);
    if !cache_file.exists() {
        return Ok(None);
    }

    let rendered = fs::read_to_string(&cache_file).map_err(|err| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("failed to read cache file {}: {err}", cache_file.display()),
        )
    })?;

    rendered.parse().map(Some).map_err(|err| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("failed to parse cache file {}: {err}", cache_file.display()),
        )
    })
}

fn write_disk_cache(cache_root: &Path, macro_name: &str, disk_key: &str, rendered: &str) {
    let cache_file = cache_file_path(cache_root, macro_name, disk_key);
    if let Some(parent) = cache_file.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(cache_file, rendered);
}

fn cache_file_path(cache_root: &Path, macro_name: &str, disk_key: &str) -> PathBuf {
    let file_name = format!("{macro_name}-{disk_key}.tok");
    cache_root.join(file_name)
}

enum CacheMode {
    Disabled,
    Disk(PathBuf),
}

fn cache_root() -> CacheMode {
    if env_flag_enabled(DISABLE_CACHE_ENV) {
        return CacheMode::Disabled;
    }

    if let Ok(path) = std::env::var(CACHE_DIR_ENV) {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return CacheMode::Disk(PathBuf::from(trimmed));
        }
    }

    warn_on_fallback_cache_dir();
    CacheMode::Disk(system_cache_root())
}

fn warn_on_fallback_cache_dir() {
    if env_flag_enabled(SILENCE_WARNING_ENV) {
        return;
    }

    FALLBACK_WARNING.call_once(|| {
        let prefix = if std::io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none() {
            "\x1b[33mwarning:\x1b[0m"
        } else {
            "warning:"
        };
        eprintln!(
            "{prefix} symbolix-compile is using the system cache directory because {CACHE_DIR_ENV} is not set; \
set {CACHE_DIR_ENV} to choose a project-specific cache path, set {SILENCE_WARNING_ENV}=1 to silence this warning, \
or set {DISABLE_CACHE_ENV}=1 to disable caching."
        );
    });
}

fn system_cache_root() -> PathBuf {
    if cfg!(target_os = "windows") {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            return Path::new(&local_app_data)
                .join("symbolix")
                .join("macro-cache");
        }
    }

    if cfg!(target_os = "macos") {
        if let Ok(home) = std::env::var("HOME") {
            return Path::new(&home)
                .join("Library")
                .join("Caches")
                .join("symbolix")
                .join("macro-cache");
        }
    }

    if let Ok(xdg_cache_home) = std::env::var("XDG_CACHE_HOME") {
        return Path::new(&xdg_cache_home)
            .join("symbolix")
            .join("macro-cache");
    }

    if let Ok(home) = std::env::var("HOME") {
        return Path::new(&home)
            .join(".cache")
            .join("symbolix")
            .join("macro-cache");
    }

    std::env::temp_dir().join("symbolix-macro-cache")
}

fn env_flag_enabled(name: &str) -> bool {
    matches!(
        std::env::var(name).ok().as_deref(),
        Some("1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON")
    )
}

fn stable_hash(value: &str) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
