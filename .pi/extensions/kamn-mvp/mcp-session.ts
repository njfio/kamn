import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { existsSync } from "node:fs";
import { resolve } from "node:path";

type Environment = Record<string, string | undefined>;
type JsonObject = Record<string, unknown>;
type PendingRequest = {
	id: string;
	resolve: (result: JsonObject) => void;
	reject: (error: Error) => void;
	timer: NodeJS.Timeout;
	removeAbort: () => void;
};
export type LiveMcpConfig = {
	binary: string;
	endpoint: string;
	agentName: string;
	keyFile: string;
	env: NodeJS.ProcessEnv;
};
export class McpSession {
	private child?: ChildProcessWithoutNullStreams;
	private pending?: PendingRequest;
	private terminalError?: Error;
	private sequence = 0;
	private stdoutBuffer = "";
	private shutdownPromise?: Promise<void>;
	private readonly config: LiveMcpConfig;
	private readonly options: { timeoutMs: number };
	constructor(config: LiveMcpConfig, options = { timeoutMs: 10000 }) {
		this.config = config;
		this.options = options;
	}
	async call(tool: string, fields: JsonObject = {}, signal?: AbortSignal): Promise<JsonObject> {
		if (signal?.aborted) throw new Error("KAMN live MCP request aborted before start");
		if (this.terminalError) throw this.terminalError;
		if (this.pending) throw new Error("KAMN live MCP session already has an active request");
		const child = this.start();
		const id = String(++this.sequence);
		return new Promise((resolveRequest, rejectRequest) => {
			this.pending = this.pendingRequest(id, resolveRequest, rejectRequest, signal);
			child.stdin.write(`${JSON.stringify({ id, tool, ...fields })}\n`, (error) => {
				if (error) this.failSession(new Error(`KAMN live MCP write failed: ${error.message}`));
			});
		});
	}
	async shutdown(): Promise<void> {
		if (this.shutdownPromise) return this.shutdownPromise;
		this.shutdownPromise = this.stopChild();
		return this.shutdownPromise;
	}
	private start(): ChildProcessWithoutNullStreams {
		if (this.child) return this.child;
		const args = ["--endpoint", this.config.endpoint, "--agent-name", this.config.agentName, "--key-file", this.config.keyFile];
		const child = spawn(this.config.binary, args, { env: this.config.env, stdio: ["pipe", "pipe", "pipe"] });
		this.child = child;
		child.stdout.setEncoding("utf8");
		child.stdout.on("data", (chunk: string) => this.consumeStdout(chunk));
		child.once("error", (error) => this.failSession(new Error(`KAMN live MCP spawn failed: ${error.message}`)));
		child.once("exit", (code, signal) => this.handleExit(code, signal));
		return child;
	}
	private pendingRequest(id: string, resolveRequest: PendingRequest["resolve"], reject: PendingRequest["reject"], signal?: AbortSignal): PendingRequest {
		const timer = setTimeout(() => this.failSession(new Error(`KAMN live MCP request ${id} timed out`)), this.options.timeoutMs);
		const abort = () => this.failSession(new Error(`KAMN live MCP request ${id} aborted`));
		signal?.addEventListener("abort", abort, { once: true });
		return { id, resolve: resolveRequest, reject, timer, removeAbort: () => signal?.removeEventListener("abort", abort) };
	}
	private consumeStdout(chunk: string) {
		this.stdoutBuffer += chunk;
		for (let newline = this.stdoutBuffer.indexOf("\n"); newline >= 0; newline = this.stdoutBuffer.indexOf("\n")) {
			const line = this.stdoutBuffer.slice(0, newline).trim();
			this.stdoutBuffer = this.stdoutBuffer.slice(newline + 1);
			if (line) this.resolveLine(line);
		}
	}
	private resolveLine(line: string) {
		let response: JsonObject;
		try {
			response = JSON.parse(line) as JsonObject;
		} catch {
			return this.failSession(new Error("KAMN live MCP returned invalid JSON"));
		}
		const pending = this.pending;
		if (!pending || response.id !== pending.id) return this.failSession(new Error("KAMN live MCP response ID mismatch"));
		this.clearPending();
		if (response.ok !== true) return pending.reject(toolError(response));
		if (!isObject(response.result)) return pending.reject(new Error("KAMN live MCP success response omitted result"));
		pending.resolve(response.result);
	}
	private handleExit(code: number | null, signal: NodeJS.Signals | null) {
		if (this.shutdownPromise) return;
		if (this.terminalError) return;
		this.failSession(new Error(`KAMN live MCP exited with code ${code ?? "none"} signal ${signal ?? "none"}`), false);
	}
	private failSession(error: Error, kill = true) {
		this.terminalError ??= error;
		const pending = this.pending;
		this.clearPending();
		pending?.reject(this.terminalError);
		if (kill) this.child?.kill("SIGTERM");
	}
	private clearPending() {
		if (!this.pending) return;
		clearTimeout(this.pending.timer);
		this.pending.removeAbort();
		this.pending = undefined;
	}
	private async stopChild(): Promise<void> {
		this.failSession(new Error("KAMN live MCP session shut down"), false);
		const child = this.child;
		if (!child || child.exitCode !== null || child.signalCode !== null) return;
		await new Promise<void>((resolveExit) => {
			child.once("exit", () => resolveExit());
			child.kill("SIGTERM");
		});
	}
}
export function readLiveMcpConfig(env: Environment = process.env, cwd = process.cwd()): LiveMcpConfig {
	const binary = absolutePath(requiredEnv(env, "KAMN_MVP_LIVE_MCP_BINARY"), cwd);
	const keyFile = absolutePath(requiredEnv(env, "KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE"), cwd);
	if (!existsSync(binary)) throw new Error(`KAMN live MCP binary does not exist: ${binary}`);
	if (!existsSync(keyFile)) throw new Error(`KAMN live MCP key file does not exist: ${keyFile}`);
	return {
		binary,
		keyFile,
		endpoint: requiredEnv(env, "KAMN_MVP_LIVE_MCP_ENDPOINT"),
		agentName: requiredEnv(env, "KAMN_MVP_LIVE_MCP_AGENT_A_NAME"),
		env: { ...process.env, ...env },
	};
}
function requiredEnv(env: Environment, name: string): string {
	const value = env[name]?.trim();
	if (!value) throw new Error(`Missing required environment variable: ${name}`);
	return value;
}
function absolutePath(path: string, cwd: string): string {
	return path.startsWith("/") ? path : resolve(cwd, path);
}
function isObject(value: unknown): value is JsonObject {
	return typeof value === "object" && value !== null && !Array.isArray(value);
}
function toolError(response: JsonObject): Error {
	const error = isObject(response.error) ? response.error : {};
	return new Error(`${String(error.kind ?? "unknown_error")}: ${String(error.message ?? "KAMN live MCP tool failed")}`);
}
