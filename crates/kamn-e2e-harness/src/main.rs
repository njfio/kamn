use kamn_e2e_harness::{
    execute_run_contract, execute_verify_contract, parse_command_args, HarnessCommand,
};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = match parse_command_args(args) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    let output = match command {
        HarnessCommand::Run(config) => execute_run_contract(&config),
        HarnessCommand::Verify(config) => execute_verify_contract(&config),
    };
    match output {
        Ok(rendered) => println!("{rendered}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
