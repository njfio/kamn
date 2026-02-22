use kamn_e2e_harness::{build_core_run_plan, ExecutionMode};

fn main() {
    let mut args = std::env::args().skip(1);
    let mut mode = ExecutionMode::SdkDirect;

    while let Some(flag) = args.next() {
        if flag == "--mode" {
            let Some(value) = args.next() else {
                eprintln!("missing value for --mode");
                std::process::exit(2);
            };
            mode = match ExecutionMode::parse(value.as_str()) {
                Ok(parsed) => parsed,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            };
        }
    }

    let plan = build_core_run_plan(mode);
    println!(
        "{{\"mode\":\"{}\",\"scenario_count\":{}}}",
        plan.mode.as_str(),
        plan.scenarios.len()
    );
}
