use std::hint::black_box;
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

use crate::model::{CaseKind, PerfCase, PerfResult, PhaseSummary, TimingStats};

pub fn benchmark_case(case: &PerfCase) -> PerfResult {
    let build = match case.kind {
        CaseKind::ApiDefault => {
            benchmark_build(case.build_iters, case.warmup_iters, case.repeat, build_semantic_from_api)
        }
        CaseKind::String => benchmark_build(case.build_iters, case.warmup_iters, case.repeat, || {
            build_semantic_from_string(&case.expression)
        }),
    };

    let semantic = match case.kind {
        CaseKind::ApiDefault => build_semantic_from_api(),
        CaseKind::String => build_semantic_from_string(&case.expression),
    };

    let compile = benchmark_compile(
        case.compile_iters,
        case.warmup_iters.min(case.compile_iters),
        case.repeat,
        &semantic,
    );
    let (execute, checksum) = benchmark_execute(
        case.exec_iters,
        case.warmup_iters.min(case.exec_iters),
        case.repeat,
        semantic,
    );

    PerfResult {
        build,
        compile,
        execute,
        checksum,
    }
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

fn benchmark_build<F>(
    iterations: usize,
    warmup_iters: usize,
    repeat: usize,
    mut build: F,
) -> PhaseSummary
where
    F: FnMut() -> SemanticExpression,
{
    for _ in 0..warmup_iters {
        black_box(build());
    }
    let mut samples = Vec::with_capacity(repeat);
    for _ in 0..repeat {
        let start = Instant::now();
        for _ in 0..iterations {
            black_box(build());
        }
        samples.push(compute_stats(iterations, start.elapsed()));
    }
    summarize_phase(samples)
}

fn benchmark_compile(
    iterations: usize,
    warmup_iters: usize,
    repeat: usize,
    semantic: &SemanticExpression,
) -> PhaseSummary {
    for _ in 0..warmup_iters {
        let compiled = jit_compile_numeric(black_box(semantic.clone()))
            .expect("failed to JIT compile semantic IR");
        black_box(compiled.arity());
    }
    let mut samples = Vec::with_capacity(repeat);
    for _ in 0..repeat {
        let start = Instant::now();
        for _ in 0..iterations {
            let compiled = jit_compile_numeric(black_box(semantic.clone()))
                .expect("failed to JIT compile semantic IR");
            black_box(compiled.arity());
        }
        samples.push(compute_stats(iterations, start.elapsed()));
    }
    summarize_phase(samples)
}

fn benchmark_execute(
    iterations: usize,
    warmup_iters: usize,
    repeat: usize,
    semantic: SemanticExpression,
) -> (PhaseSummary, f64) {
    let compiled = jit_compile_numeric(semantic).expect("failed to JIT compile semantic IR");
    let inputs = generate_inputs(iterations, compiled.arity());

    let warmup_count = warmup_iters.min(inputs.len());
    for values in &inputs[..warmup_count] {
        black_box(
            compiled.calculate_unchecked(black_box(values)),
        );
    }

    let mut checksum = 0.0;
    let mut samples = Vec::with_capacity(repeat);
    for run in 0..repeat {
        let start = Instant::now();
        let mut run_checksum = 0.0;
        for values in &inputs {
            run_checksum += compiled.calculate_unchecked(black_box(values));
        }
        if run == 0 {
            checksum = run_checksum;
        }
        samples.push(compute_stats(iterations, start.elapsed()));
    }

    (summarize_phase(samples), checksum)
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

fn summarize_phase(samples: Vec<TimingStats>) -> PhaseSummary {
    let sample_count = samples.len();
    let min = samples
        .iter()
        .min_by(|a, b| a.total_ms.partial_cmp(&b.total_ms).unwrap())
        .cloned()
        .expect("phase must have at least one sample");

    let mean = TimingStats {
        total_ms: samples.iter().map(|s| s.total_ms).sum::<f64>() / sample_count as f64,
        avg_ns: samples.iter().map(|s| s.avg_ns).sum::<f64>() / sample_count as f64,
        iter_s: samples.iter().map(|s| s.iter_s).sum::<f64>() / sample_count as f64,
    };

    PhaseSummary {
        samples: sample_count,
        mean,
        min,
    }
}
