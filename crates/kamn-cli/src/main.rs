use kamn_cli::{dispatch, parse_cli_args, OutputFormat};

fn main() {
    let parsed = match parse_cli_args(std::env::args()) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("kamn-cli parse error: {error}");
            std::process::exit(2);
        }
    };

    match dispatch(&parsed) {
        Ok(output) => match parsed.output_format {
            OutputFormat::Json => println!("{{\"status\":\"ok\",\"result\":\"{}\"}}", output),
            OutputFormat::Text => println!("{output}"),
        },
        Err(error) => {
            eprintln!("kamn-cli command error: {error}");
            std::process::exit(1);
        }
    }
}
