use std::path::Path;

use crate::model::{CaseFile, PerfCase, PerfResult, PerfResultCompat, TimingStats};

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

pub fn compare_best_and_print(best: &CaseFile, current: &CaseFile, best_path: &Path) {
    println!("comparison against best {}", best_path.display());
    println!();

    for case in &current.cases {
        let Some(current_result) = case.result.as_ref().map(PerfResultCompat::as_current) else {
            continue;
        };
        let Some(best_case) = best.cases.iter().find(|prev| prev.name == case.name) else {
            println!("{}: no best record yet", case.name);
            continue;
        };
        let Some(best_result) = best_case.result.as_ref().map(PerfResultCompat::as_current) else {
            println!("{}: best record has no result", case.name);
            continue;
        };

        println!("{}", case.name);
        print_delta("build mean", best_result.build.mean.avg_ns, current_result.build.mean.avg_ns);
        print_delta("build min ", best_result.build.min.avg_ns, current_result.build.min.avg_ns);
        print_delta(
            "compile mean",
            best_result.compile.mean.avg_ns,
            current_result.compile.mean.avg_ns,
        );
        print_delta(
            "compile min ",
            best_result.compile.min.avg_ns,
            current_result.compile.min.avg_ns,
        );
        print_delta(
            "execute mean",
            best_result.execute.mean.avg_ns,
            current_result.execute.mean.avg_ns,
        );
        print_delta(
            "execute min ",
            best_result.execute.min.avg_ns,
            current_result.execute.min.avg_ns,
        );
        println!();
    }
}

pub fn merge_best_results(best: &mut CaseFile, current: &CaseFile) -> bool {
    let mut changed = false;

    for current_case in &current.cases {
        let Some(current_result) = current_case.result.as_ref().map(PerfResultCompat::as_current) else {
            continue;
        };

        match best.cases.iter_mut().find(|case| case.name == current_case.name) {
            Some(best_case) => {
                let should_replace = match best_case.result.as_ref().map(PerfResultCompat::as_current) {
                    Some(best_result) => is_better_result(&current_result, &best_result),
                    None => true,
                };
                if should_replace {
                    best_case.kind = current_case.kind.clone();
                    best_case.expression = current_case.expression.clone();
                    best_case.build_iters = current_case.build_iters;
                    best_case.compile_iters = current_case.compile_iters;
                    best_case.exec_iters = current_case.exec_iters;
                    best_case.warmup_iters = current_case.warmup_iters;
                    best_case.repeat = current_case.repeat;
                    best_case.result = Some(PerfResultCompat::Current(current_result));
                    changed = true;
                }
            }
            None => {
                best.cases.push(current_case.clone());
                changed = true;
            }
        }
    }

    changed
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

fn is_better_result(current: &PerfResult, best: &PerfResult) -> bool {
    compare_stats(&current.execute.mean, &best.execute.mean)
        .then_with(|| compare_stats(&current.execute.min, &best.execute.min))
        .then_with(|| compare_stats(&current.compile.mean, &best.compile.mean))
        .then_with(|| compare_stats(&current.build.mean, &best.build.mean))
        .then_with(|| compare_stats(&current.compile.min, &best.compile.min))
        .then_with(|| compare_stats(&current.build.min, &best.build.min))
        .is_lt()
}

fn compare_stats(current: &TimingStats, best: &TimingStats) -> std::cmp::Ordering {
    current
        .avg_ns
        .partial_cmp(&best.avg_ns)
        .unwrap_or(std::cmp::Ordering::Equal)
}
