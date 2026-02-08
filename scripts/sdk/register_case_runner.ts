import { KAMNClient, SDKError } from "../../packages/kamn-sdk/src/index.ts";

type ParsedArgs = {
  agentType: string;
  modelFamily: string;
  capabilities: string[];
};

function usage(): never {
  throw new Error(
    "usage: register_case_runner.ts --agent-type <value> --model-family <value> [--capability <value>]...",
  );
}

function parseArgs(argv: string[]): ParsedArgs {
  let agentType = "";
  let modelFamily = "";
  const capabilities: string[] = [];

  for (let index = 0; index < argv.length; ) {
    const arg = argv[index];
    switch (arg) {
      case "--agent-type":
        if (index + 1 >= argv.length) {
          usage();
        }
        agentType = argv[index + 1] ?? "";
        index += 2;
        break;
      case "--model-family":
        if (index + 1 >= argv.length) {
          usage();
        }
        modelFamily = argv[index + 1] ?? "";
        index += 2;
        break;
      case "--capability":
        if (index + 1 >= argv.length) {
          usage();
        }
        capabilities.push(argv[index + 1] ?? "");
        index += 2;
        break;
      default:
        usage();
    }
  }

  return { agentType, modelFamily, capabilities };
}

function sanitize(value: string): string {
  return value.replaceAll("\n", " ");
}

const parsed = parseArgs(process.argv.slice(2));
if (!parsed.agentType && !parsed.modelFamily && parsed.capabilities.length === 0) {
  usage();
}

const client = new KAMNClient();
try {
  const did = client.register(
    parsed.agentType,
    parsed.modelFamily,
    parsed.capabilities,
  );
  console.log("status=ok");
  console.log(`did=${did}`);
} catch (error: unknown) {
  if (error instanceof SDKError) {
    console.log("status=error");
    console.log(`error=${sanitize(error.message)}`);
  } else if (error instanceof Error) {
    console.log("status=error");
    console.log(`error=${sanitize(error.message)}`);
  } else {
    console.log("status=error");
    console.log("error=unknown error");
  }
}
