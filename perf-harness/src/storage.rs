use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::{default_case_file, CaseFile};

pub const INPUT_DIR: &str = "input";
pub const OUTPUT_DIR: &str = "output";
pub const DEFAULT_INPUT_FILE: &str = "perf_cases.toml";

pub fn resolve_input_path(arg: Option<String>) -> PathBuf {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join(INPUT_DIR);
    match arg {
        Some(path) => {
            let candidate = Path::new(&path);
            if candidate.is_absolute() {
                candidate.to_path_buf()
            } else {
                base.join(candidate)
            }
        }
        None => base.join(DEFAULT_INPUT_FILE),
    }
}

pub fn ensure_case_file(path: &Path) -> io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let cases = default_case_file();
    save_cases(path, &cases)
}

pub fn load_cases(path: &Path) -> io::Result<CaseFile> {
    let content = fs::read_to_string(path)?;
    toml::from_str(&content).map_err(|err| invalid_case_file(path, &err.to_string()))
}

pub fn save_cases(path: &Path, cases: &CaseFile) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content =
        toml::to_string_pretty(cases).map_err(|err| invalid_case_file(path, &err.to_string()))?;
    fs::write(path, content)
}

pub fn next_output_path(input_path: &Path) -> io::Result<(PathBuf, Option<PathBuf>)> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output_dir = manifest_dir.join(OUTPUT_DIR);
    fs::create_dir_all(&output_dir)?;

    let stem = input_path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| invalid_case_file(input_path, "input file must have a valid stem"))?;
    let ext = input_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("toml");

    let previous = find_latest_output(&output_dir, stem, ext)?;
    let mut timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?
        .as_secs();

    loop {
        let candidate = output_dir.join(format!("{stem}_{timestamp}.{ext}"));
        if !candidate.exists() {
            return Ok((candidate, previous));
        }
        timestamp += 1;
    }
}

pub fn best_output_path(input_path: &Path) -> io::Result<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output_dir = manifest_dir.join(OUTPUT_DIR);
    fs::create_dir_all(&output_dir)?;

    let stem = input_path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| invalid_case_file(input_path, "input file must have a valid stem"))?;
    let ext = input_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("toml");

    Ok(output_dir.join(format!("{stem}_best.{ext}")))
}

pub fn invalid_case_file(path: &Path, detail: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("invalid case file {}: {detail}", path.display()),
    )
}

fn find_latest_output(output_dir: &Path, stem: &str, ext: &str) -> io::Result<Option<PathBuf>> {
    let prefix = format!("{stem}_");
    let suffix = format!(".{ext}");
    let mut latest: Option<(SystemTime, PathBuf)> = None;

    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !file_name.starts_with(&prefix) || !file_name.ends_with(&suffix) {
            continue;
        }

        let modified = entry
            .metadata()?
            .modified()
            .unwrap_or(UNIX_EPOCH);

        match &latest {
            Some((current_modified, _)) if modified <= *current_modified => {}
            _ => latest = Some((modified, path)),
        }
    }

    Ok(latest.map(|(_, path)| path))
}
