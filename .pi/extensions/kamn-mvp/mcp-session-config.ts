import { existsSync } from "node:fs";
import { resolve } from "node:path";

type Environment = Record<string, string | undefined>;
export type LiveMcpAgent = "AGENT_A" | "AGENT_B" | "AGENT_C";
export type LiveMcpConfig = {
	binary: string;
	endpoint: string;
	agentName: string;
	keyFile: string;
	requestTimeoutMs: number;
	env: NodeJS.ProcessEnv;
};

const PROCESS_ENV_ALLOWLIST = new Set([
	"HOME", "PATH", "RUST_LOG", "TMPDIR", "KAMN_SDK_SERVICE_TIMEOUT_SECONDS",
]);
const FIXTURE_ENV_ALLOWLIST = new Set([
	"KAMN_MVP_FAKE_MCP_MODE", "KAMN_MVP_FAKE_MCP_RESULT_MODE",
	"KAMN_MVP_FAKE_MCP_START_FILE", "KAMN_MVP_FAKE_MCP_STOP_FILE",
]);
const AGENT_ENV = {
	AGENT_A: { name: "KAMN_MVP_LIVE_MCP_AGENT_A_NAME", keyFile: "KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE" },
	AGENT_B: { name: "KAMN_MVP_LIVE_MCP_AGENT_B_NAME", keyFile: "KAMN_MVP_LIVE_MCP_AGENT_B_KEY_FILE" },
	AGENT_C: { name: "KAMN_MVP_LIVE_MCP_AGENT_C_NAME", keyFile: "KAMN_MVP_LIVE_MCP_AGENT_C_KEY_FILE" },
} as const;

export function readLiveMcpConfig(agent: LiveMcpAgent, env: Environment = process.env, cwd = process.cwd()): LiveMcpConfig {
	const agentEnv = AGENT_ENV[agent];
	const binary = absolutePath(requiredEnv(env, "KAMN_MVP_LIVE_MCP_BINARY"), cwd);
	const keyFile = absolutePath(requiredEnv(env, agentEnv.keyFile), cwd);
	if (!existsSync(binary)) throw new Error(`KAMN live MCP binary does not exist: ${binary}`);
	if (!existsSync(keyFile)) throw new Error(`KAMN live MCP key file does not exist: ${keyFile}`);
	return {
		binary, keyFile,
		endpoint: requiredEnv(env, "KAMN_MVP_LIVE_MCP_ENDPOINT"),
		agentName: requiredEnv(env, agentEnv.name),
		requestTimeoutMs: requestTimeoutMs(env),
		env: childEnvironment(env),
	};
}

function requestTimeoutMs(env: Environment): number {
	const raw = env.KAMN_SDK_SERVICE_TIMEOUT_SECONDS;
	if (raw === undefined) return 10000;
	const seconds = Number(raw.trim());
	const milliseconds = seconds * 1000;
	if (!/^\d+$/.test(raw.trim()) || seconds <= 0 || !Number.isSafeInteger(milliseconds)) {
		throw new Error("KAMN_SDK_SERVICE_TIMEOUT_SECONDS must be a positive integer");
	}
	return milliseconds;
}

function childEnvironment(env: Environment): NodeJS.ProcessEnv {
	return Object.fromEntries(Object.entries({ ...process.env, ...env }).filter(
		([name, value]) => value !== undefined && (PROCESS_ENV_ALLOWLIST.has(name) || FIXTURE_ENV_ALLOWLIST.has(name)),
	));
}

function requiredEnv(env: Environment, name: string): string {
	const value = env[name]?.trim();
	if (!value) throw new Error(`Missing required environment variable: ${name}`);
	return value;
}

function absolutePath(path: string, cwd: string): string {
	return path.startsWith("/") ? path : resolve(cwd, path);
}
