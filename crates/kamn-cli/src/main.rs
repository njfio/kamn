use kamn_cli::{dispatch, parse_cli_args, OutputFormat};

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if kamn_cli::is_help_request(args.iter().map(String::as_str)) {
        println!("{}", kamn_cli::render_help_text());
        return;
    }

    let parsed = match parse_cli_args(args) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("kamn-cli parse error: {error}");
            std::process::exit(2);
        }
    };

    match dispatch(&parsed) {
        Ok(output) => match parsed.output_format {
            OutputFormat::Json => println!("{}", output.json),
            OutputFormat::Text => println!("{}", output.text),
        },
        Err(error) => {
            eprintln!("kamn-cli command error: {error}");
            std::process::exit(1);
        }
    }
}
