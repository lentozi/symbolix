use serde::{Deserialize, Serialize};

pub const DEFAULT_BUILD_ITERS: usize = 20_000;
pub const DEFAULT_COMPILE_ITERS: usize = 2_000;
pub const DEFAULT_EXEC_ITERS: usize = 1_000_000;
pub const DEFAULT_WARMUP_ITERS: usize = 1_000;
pub const DEFAULT_REPEAT: usize = 5;
pub const DEFAULT_API_EXPR: &str =
    "(((x + 1.25) ^ 2 + y * 3.5 - z / 7.0) * (x - y + 2.0)) / 3.0";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CaseFile {
    pub cases: Vec<PerfCase>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerfCase {
    pub name: String,
    pub kind: CaseKind,
    pub expression: String,
    #[serde(default = "default_build_iters")]
    pub build_iters: usize,
    #[serde(default = "default_compile_iters")]
    pub compile_iters: usize,
    #[serde(default = "default_exec_iters")]
    pub exec_iters: usize,
    #[serde(default = "default_warmup_iters")]
    pub warmup_iters: usize,
    #[serde(default = "default_repeat")]
    pub repeat: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<PerfResultCompat>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseKind {
    ApiDefault,
    String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerfResult {
    pub build: PhaseSummary,
    pub compile: PhaseSummary,
    pub execute: PhaseSummary,
    pub checksum: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PerfResultCompat {
    Current(PerfResult),
    Legacy(LegacyPerfResult),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LegacyPerfResult {
    pub build: TimingStats,
    pub compile: TimingStats,
    pub execute: TimingStats,
    pub checksum: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimingStats {
    pub total_ms: f64,
    pub avg_ns: f64,
    pub iter_s: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhaseSummary {
    pub samples: usize,
    pub mean: TimingStats,
    pub min: TimingStats,
}

pub fn default_build_iters() -> usize {
    DEFAULT_BUILD_ITERS
}

pub fn default_compile_iters() -> usize {
    DEFAULT_COMPILE_ITERS
}

pub fn default_exec_iters() -> usize {
    DEFAULT_EXEC_ITERS
}

pub fn default_warmup_iters() -> usize {
    DEFAULT_WARMUP_ITERS
}

pub fn default_repeat() -> usize {
    DEFAULT_REPEAT
}

impl CaseKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            CaseKind::ApiDefault => "api_default",
            CaseKind::String => "string",
        }
    }
}

impl PerfResultCompat {
    pub fn as_current(&self) -> PerfResult {
        match self {
            PerfResultCompat::Current(result) => result.clone(),
            PerfResultCompat::Legacy(result) => PerfResult {
                build: PhaseSummary {
                    samples: 1,
                    mean: result.build.clone(),
                    min: result.build.clone(),
                },
                compile: PhaseSummary {
                    samples: 1,
                    mean: result.compile.clone(),
                    min: result.compile.clone(),
                },
                execute: PhaseSummary {
                    samples: 1,
                    mean: result.execute.clone(),
                    min: result.execute.clone(),
                },
                checksum: result.checksum,
            },
        }
    }
}

pub fn default_case_file() -> CaseFile {
    CaseFile {
        cases: vec![
            PerfCase {
                name: "api_default_expr".to_string(),
                kind: CaseKind::ApiDefault,
                expression: DEFAULT_API_EXPR.to_string(),
                build_iters: DEFAULT_BUILD_ITERS,
                compile_iters: DEFAULT_COMPILE_ITERS,
                exec_iters: DEFAULT_EXEC_ITERS,
                warmup_iters: DEFAULT_WARMUP_ITERS,
                repeat: DEFAULT_REPEAT,
                result: None,
            },
            PerfCase {
                name: "string_default_expr".to_string(),
                kind: CaseKind::String,
                expression: DEFAULT_API_EXPR.to_string(),
                build_iters: DEFAULT_BUILD_ITERS,
                compile_iters: DEFAULT_COMPILE_ITERS,
                exec_iters: DEFAULT_EXEC_ITERS,
                warmup_iters: DEFAULT_WARMUP_ITERS,
                repeat: DEFAULT_REPEAT,
                result: None,
            },
        ],
    }
}
