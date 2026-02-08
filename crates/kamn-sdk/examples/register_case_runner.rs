use kamn_sdk::{AgentMetadata, InMemoryKamnClient, KamnAgent};

fn usage() -> ! {
    eprintln!(
        "usage: register_case_runner --agent-type <value> --model-family <value> [--capability <value>]..."
    );
    std::process::exit(2);
}

fn sanitize(value: &str) -> String {
    value.replace('\n', " ")
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut agent_type = None::<String>;
    let mut model_family = None::<String>;
    let mut capabilities = Vec::<String>::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--agent-type" => {
                let value = args.next().unwrap_or_else(|| usage());
                agent_type = Some(value);
            }
            "--model-family" => {
                let value = args.next().unwrap_or_else(|| usage());
                model_family = Some(value);
            }
            "--capability" => {
                let value = args.next().unwrap_or_else(|| usage());
                capabilities.push(value);
            }
            _ => usage(),
        }
    }

    let metadata = AgentMetadata {
        agent_type: agent_type.unwrap_or_else(|| usage()),
        model_family: model_family.unwrap_or_else(|| usage()),
        capabilities,
    };

    let mut client = InMemoryKamnClient::new();
    match client.register(metadata) {
        Ok(did) => {
            println!("status=ok");
            println!("did={}", did.as_str());
        }
        Err(error) => {
            println!("status=error");
            println!("error={}", sanitize(&error.to_string()));
        }
    }
}
