use std::fs;
use std::hint::black_box;
use std::path::Path;
use std::time::{Duration, Instant};

use exprion_api::{scope, Var};
use exprion_core::{
    lexer::Lexer,
    new_compile_context,
    optimizer::optimize,
    parser::Parser,
    semantic::{semantic_ir::SemanticExpression, Analyzer},
};
use exprion_engine::jit_compile_numeric;
use serde::{Deserialize, Serialize};

const CASE_FILE: &str = "perf_cases.toml";
const DEFAULT_BUILD_ITERS: usize = 20_000;
const DEFAULT_COMPILE_ITERS: usize = 2_000;
const DEFAULT_EXEC_ITERS: usize = 1_000_000;
const DEFAULT_API_EXPR: &str = "(((x + 1.25) ^ 2 + y * 3.5 - z / 7.0) * (x - y + 2.0)) / 3.0";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CaseFile {
    cases: Vec<PerfCase>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PerfCase {
    name: String,
    kind: CaseKind,
    expression: String,
    #[serde(default = "default_build_iters")]
    build_iters: usize,
    #[serde(default = "default_compile_iters")]
    compile_iters: usize,
    #[serde(default = "default_exec_iters")]
    exec_iters: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<PerfResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CaseKind {
    ApiDefault,
    String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PerfResult {
    build: TimingStats,
    compile: TimingStats,
    execute: TimingStats,
    checksum: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TimingStats {
    total_ms: f64,
    avg_ns: f64,
    iter_s: f64,
}

fn default_build_iters() -> usize {
    DEFAULT_BUILD_ITERS
}

fn default_compile_iters() -> usize {
    DEFAULT_COMPILE_ITERS
}

fn default_exec_iters() -> usize {
    DEFAULT_EXEC_ITERS
}

fn build_semantic_from_string(expression: &str) -> SemanticExpression {
    new_compile_context! {
        let mut lexer = Lexer::new(expression);
        let parsed = Parser::pratt(&mut lexer);
        let mut analyzer = Analyzer::new();
        let mut semantic = analyzer.analyze_with_ctx(&parsed);
        optimize(&mut semantic);
        semantic
    }
}

fn build_semantic_from_api() -> SemanticExpression {
    scope(|| {
        let x = Var::number("x");
        let y = Var::number("y");
        let z = Var::number("z");

        let expr = (((&x + 1.25).pow(2.0) + &y * 3.5 - &z / 7.0) * (&x - &y + 2.0)) / 3.0;
        let mut semantic = expr.into_semantic();
        optimize(&mut semantic);
        semantic
    })
}

fn benchmark_case(case: &PerfCase) -> PerfResult {
    let build = match case.kind {
        CaseKind::ApiDefault => benchmark_build(case.build_iters, build_semantic_from_api),
        CaseKind::String => benchmark_build(case.build_iters, || {
            build_semantic_from_string(&case.expression)
        }),
    };

    let semantic = match case.kind {
        CaseKind::ApiDefault => build_semantic_from_api(),
        CaseKind::String => build_semantic_from_string(&case.expression),
    };

    let compile = benchmark_compile(case.compile_iters, &semantic);
    let (execute, checksum) = benchmark_execute(case.exec_iters, semantic);

    PerfResult {
        build,
        compile,
        execute,
        checksum,
    }
}

fn benchmark_build<F>(iterations: usize, mut build: F) -> TimingStats
where
    F: FnMut() -> SemanticExpression,
{
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(build());
    }
    compute_stats(iterations, start.elapsed())
}

fn benchmark_compile(iterations: usize, semantic: &SemanticExpression) -> TimingStats {
    let start = Instant::now();
    for _ in 0..iterations {
        let compiled = jit_compile_numeric(black_box(semantic.clone()))
            .expect("failed to JIT compile semantic IR");
        black_box(compiled.arity());
    }
    compute_stats(iterations, start.elapsed())
}

fn benchmark_execute(iterations: usize, semantic: SemanticExpression) -> (TimingStats, f64) {
    let compiled = jit_compile_numeric(semantic).expect("failed to JIT compile semantic IR");
    let inputs = generate_inputs(iterations, compiled.arity());
    let start = Instant::now();
    let mut checksum = 0.0;

    for values in &inputs {
        checksum += compiled
            .calculate(black_box(values))
            .expect("failed to execute JIT function");
    }

    (compute_stats(iterations, start.elapsed()), checksum)
}

fn generate_inputs(iterations: usize, arity: usize) -> Vec<Vec<f64>> {
    let mut inputs = Vec::with_capacity(iterations);
    for i in 0..iterations {
        let mut row = Vec::with_capacity(arity);
        for j in 0..arity {
            let base = 1.0 + j as f64;
            let period = 53 + j * 17;
            let step = 0.001 * (j as f64 + 1.0);
            row.push(base + (i % period) as f64 * step);
        }
        inputs.push(row);
    }
    inputs
}

fn compute_stats(iterations: usize, elapsed: Duration) -> TimingStats {
    let secs = elapsed.as_secs_f64();
    TimingStats {
        total_ms: secs * 1000.0,
        avg_ns: secs * 1_000_000_000.0 / iterations as f64,
        iter_s: iterations as f64 / secs,
    }
}

fn load_cases(path: &Path) -> std::io::Result<CaseFile> {
    let content = fs::read_to_string(path)?;
    toml::from_str(&content).map_err(|err| invalid_case_file(path, &err.to_string()))
}

fn save_cases(path: &Path, cases: &CaseFile) -> std::io::Result<()> {
    let content = toml::to_string_pretty(cases)
        .map_err(|err| invalid_case_file(path, &err.to_string()))?;
    fs::write(path, content)
}

fn invalid_case_file(path: &Path, detail: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!("invalid case file {}: {detail}", path.display()),
    )
}

fn print_case(case: &PerfCase, result: &PerfResult) {
    println!("{}", case.name);
    println!("  kind: {}", case.kind.as_str());
    println!(
        "  build: {:.3} ms total, {:.1} ns/iter, {:.0} iter/s",
        result.build.total_ms, result.build.avg_ns, result.build.iter_s
    );
    println!(
        "  compile: {:.3} ms total, {:.1} ns/iter, {:.0} iter/s",
        result.compile.total_ms, result.compile.avg_ns, result.compile.iter_s
    );
    println!(
        "  execute: {:.3} ms total, {:.1} ns/iter, {:.0} iter/s",
        result.execute.total_ms, result.execute.avg_ns, result.execute.iter_s
    );
    println!("  checksum: {:.6}", result.checksum);
}

impl CaseKind {
    fn as_str(&self) -> &'static str {
        match self {
            CaseKind::ApiDefault => "api_default",
            CaseKind::String => "string",
        }
    }
}

fn ensure_case_file(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        return Ok(());
    }

    let cases = CaseFile {
        cases: vec![
            PerfCase {
                name: "api_default_expr".to_string(),
                kind: CaseKind::ApiDefault,
                expression: DEFAULT_API_EXPR.to_string(),
                build_iters: DEFAULT_BUILD_ITERS,
                compile_iters: DEFAULT_COMPILE_ITERS,
                exec_iters: DEFAULT_EXEC_ITERS,
                result: None,
            },
            PerfCase {
                name: "string_default_expr".to_string(),
                kind: CaseKind::String,
                expression: DEFAULT_API_EXPR.to_string(),
                build_iters: DEFAULT_BUILD_ITERS,
                compile_iters: DEFAULT_COMPILE_ITERS,
                exec_iters: DEFAULT_EXEC_ITERS,
                result: None,
            },
        ],
    };
    save_cases(path, &cases)
}

fn main() {
    let case_path_buf = Path::new(env!("CARGO_MANIFEST_DIR")).join(CASE_FILE);
    let case_path = case_path_buf.as_path();
    ensure_case_file(case_path).expect("failed to create perf case file");

    let mut case_file = load_cases(case_path).expect("failed to load perf cases");
    println!("exprion performance harness");
    println!("case file: {}", case_path.display());
    println!();

    for case in &mut case_file.cases {
        let result = benchmark_case(case);
        print_case(case, &result);
        println!();
        case.result = Some(result);
    }

    save_cases(case_path, &case_file).expect("failed to write perf results back to case file");
    println!("results written back to {}", case_path.display());
}
