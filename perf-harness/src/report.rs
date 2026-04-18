use std::path::Path;

use crate::model::{CaseFile, PerfCase, PerfResult, PerfResultCompat};

pub fn print_case(case: &PerfCase, result: &PerfResult) {
    println!("{}", case.name);
    println!("  kind: {}", case.kind.as_str());
    println!(
        "  config: warmup={} repeat={} build_iters={} compile_iters={} exec_iters={}",
        case.warmup_iters, case.repeat, case.build_iters, case.compile_iters, case.exec_iters
    );
    println!(
        "  build mean: {:.3} ms total, {:.1} ns/iter, {:.0} iter/s",
        result.build.mean.total_ms, result.build.mean.avg_ns, result.build.mean.iter_s
    );
    println!(
        "  build min : {:.3} ms total, {:.1} ns/iter, {:.0} iter/s",
        result.build.min.total_ms, result.build.min.avg_ns, result.build.min.iter_s
    );
    println!(
        "  compile mean: {:.3} ms total, {:.1} ns/iter, {:.0} iter/s",
        result.compile.mean.total_ms, result.compile.mean.avg_ns, result.compile.mean.iter_s
    );
    println!(
        "  compile min : {:.3} ms total, {:.1} ns/iter, {:.0} iter/s",
        result.compile.min.total_ms, result.compile.min.avg_ns, result.compile.min.iter_s
    );
    println!(
        "  execute mean: {:.3} ms total, {:.1} ns/iter, {:.0} iter/s",
        result.execute.mean.total_ms, result.execute.mean.avg_ns, result.execute.mean.iter_s
    );
    println!(
        "  execute min : {:.3} ms total, {:.1} ns/iter, {:.0} iter/s",
        result.execute.min.total_ms, result.execute.min.avg_ns, result.execute.min.iter_s
    );
    println!("  checksum: {:.6}", result.checksum);
}

pub fn compare_and_print(previous: &CaseFile, current: &CaseFile, previous_path: &Path) {
    println!("comparison against {}", previous_path.display());
    println!();

    for case in &current.cases {
        let Some(current_result) = case.result.as_ref().map(PerfResultCompat::as_current) else {
            continue;
        };
        let Some(previous_case) = previous.cases.iter().find(|prev| prev.name == case.name) else {
            println!("{}: no previous case with the same name", case.name);
            continue;
        };
        let Some(previous_result) = previous_case
            .result
            .as_ref()
            .map(PerfResultCompat::as_current)
        else {
            println!("{}: previous case has no result", case.name);
            continue;
        };

        println!("{}", case.name);
        print_delta(
            "build mean",
            previous_result.build.mean.avg_ns,
            current_result.build.mean.avg_ns,
        );
        print_delta(
            "build min ",
            previous_result.build.min.avg_ns,
            current_result.build.min.avg_ns,
        );
        print_delta(
            "compile mean",
            previous_result.compile.mean.avg_ns,
            current_result.compile.mean.avg_ns,
        );
        print_delta(
            "compile min ",
            previous_result.compile.min.avg_ns,
            current_result.compile.min.avg_ns,
        );
        print_delta(
            "execute mean",
            previous_result.execute.mean.avg_ns,
            current_result.execute.mean.avg_ns,
        );
        print_delta(
            "execute min ",
            previous_result.execute.min.avg_ns,
            current_result.execute.min.avg_ns,
        );
        println!();
    }
}

fn print_delta(label: &str, previous: f64, current: f64) {
    let delta = current - previous;
    let percent = if previous.abs() > f64::EPSILON {
        delta / previous * 100.0
    } else {
        0.0
    };
    println!(
        "  {label}: prev {:.1} ns, curr {:.1} ns, delta {:+.1} ns ({:+.2}%)",
        previous, current, delta, percent
    );
}
