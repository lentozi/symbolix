mod bench;
mod model;
mod report;
mod storage;

use std::env;

use bench::benchmark_case;
use model::PerfResultCompat;
use report::{compare_and_print, compare_best_and_print, merge_best_results, print_case};
use storage::{
    best_output_path, ensure_case_file, load_cases, next_output_path, resolve_input_path,
    save_cases,
};

fn main() {
    let input_path = resolve_input_path(env::args().nth(1));
    ensure_case_file(&input_path).expect("failed to create perf case file");

    let mut case_file = load_cases(&input_path).expect("failed to load perf cases");
    let (output_path, previous_output) =
        next_output_path(&input_path).expect("failed to determine output file path");
    let best_path = best_output_path(&input_path).expect("failed to determine best output file");

    println!("exprion performance harness");
    println!("input file : {}", input_path.display());
    println!("output file: {}", output_path.display());
    println!();

    for case in &mut case_file.cases {
        let result = benchmark_case(case);
        print_case(case, &result);
        println!();
        case.result = Some(PerfResultCompat::Current(result));
    }

    save_cases(&output_path, &case_file).expect("failed to write perf results to output file");
    println!("results written to {}", output_path.display());

    let mut best_file = if best_path.exists() {
        load_cases(&best_path).expect("failed to load best perf output")
    } else {
        model::CaseFile { cases: Vec::new() }
    };
    let best_updated = merge_best_results(&mut best_file, &case_file);
    if best_updated || !best_path.exists() {
        save_cases(&best_path, &best_file).expect("failed to write best perf output");
    }
    println!("best file: {}", best_path.display());

    if let Some(previous_path) = previous_output {
        println!();
        let previous = load_cases(&previous_path).expect("failed to load previous perf output");
        compare_and_print(&previous, &case_file, &previous_path);
    } else {
        println!();
        println!("no previous round found for comparison");
    }

    println!();
    compare_best_and_print(&best_file, &case_file, &best_path);
}
