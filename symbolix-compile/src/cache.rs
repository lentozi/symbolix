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

fn parse_cached_tokens(rendered: &str, cache_file: &Path) -> syn::Result<TokenStream> {
    rendered.parse().map_err(|err| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("failed to parse cache file {}: {err}", cache_file.display()),
        )
    })
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

    parse_cached_tokens(&rendered, &cache_file).map(Some)
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

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream as TokenStream2;
    use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("symbolix_compile_cache_unit_{unique}"))
    }

    #[test]
    fn stable_hash_is_deterministic() {
        assert_eq!(stable_hash("symbolix"), stable_hash("symbolix"));
        assert_ne!(stable_hash("symbolix"), stable_hash("formula"));
    }

    #[test]
    fn env_flag_enabled_accepts_expected_values() {
        for value in ["1", "true", "TRUE", "yes", "YES", "on", "ON"] {
            std::env::set_var("SYMBOLIX_TEST_FLAG", value);
            assert!(env_flag_enabled("SYMBOLIX_TEST_FLAG"));
        }
        std::env::set_var("SYMBOLIX_TEST_FLAG", "0");
        assert!(!env_flag_enabled("SYMBOLIX_TEST_FLAG"));
        std::env::remove_var("SYMBOLIX_TEST_FLAG");
    }

    #[test]
    fn cache_file_path_and_disk_roundtrip_work() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).unwrap();
        let cache_path = cache_file_path(&dir, "formula", "abc");
        assert!(cache_path.ends_with("formula-abc.tok"));

        write_disk_cache(&dir, "formula", "abc", "1 + 2");
        let rendered = fs::read_to_string(cache_path).unwrap();
        let reparsed: TokenStream2 = rendered.parse().unwrap();
        let rendered = reparsed.to_string();
        assert!(rendered.contains("1"));
        assert!(rendered.contains("2"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_disk_cache_reports_parse_errors() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).unwrap();
        let path = cache_file_path(&dir, "formula", "bad");
        fs::write(&path, "(").unwrap();

        let rendered = fs::read_to_string(&path).unwrap();
        let err = rendered.parse::<TokenStream2>().unwrap_err();
        assert!(!err.to_string().is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cache_root_respects_env_configuration() {
        std::env::set_var(DISABLE_CACHE_ENV, "1");
        assert!(matches!(cache_root(), CacheMode::Disabled));
        std::env::remove_var(DISABLE_CACHE_ENV);

        let dir = temp_dir();
        std::env::set_var(CACHE_DIR_ENV, &dir);
        match cache_root() {
            CacheMode::Disk(path) => assert_eq!(path, dir),
            CacheMode::Disabled => panic!("cache should not be disabled"),
        }
        std::env::remove_var(CACHE_DIR_ENV);
    }

    #[test]
    fn write_disk_cache_persists_rendered_tokens() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).unwrap();
        write_disk_cache(&dir, "formula", "hash", "3 + 4");
        let rendered = fs::read_to_string(cache_file_path(&dir, "formula", "hash")).unwrap();
        assert_eq!(rendered, "3 + 4");
        let _ = fs::remove_dir_all(&dir);
    }
}
