import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { McpProvenanceTracker, type McpSessionProvenance } from "./mcp-provenance.ts";
import { validateAuthority } from "./mcp-authority.ts";
import type { LiveMcpConfig } from "./mcp-session-config.ts";
export { readLiveMcpConfig } from "./mcp-session-config.ts";
export type { LiveMcpAgent } from "./mcp-session-config.ts";
export type { McpSessionProvenance } from "./mcp-provenance.ts";

type JsonObject = Record<string, unknown>;
type PendingRequest = {
	id: string;
	tool: string;
	resolve: (result: JsonObject) => void;
	reject: (error: Error) => void;
	timer: NodeJS.Timeout;
	removeAbort: () => void;
};
export class McpSession {
	private child?: ChildProcessWithoutNullStreams;
	private pending?: PendingRequest;
	private terminalError?: Error;
	private sequence = 0;
	private registeredActor?: string;
	private readonly provenanceTracker = new McpProvenanceTracker();
	private stdoutBuffer = "";
	private shutdownPromise?: Promise<void>;
	private readonly config: LiveMcpConfig;
	private readonly options: { timeoutMs: number };
	constructor(config: LiveMcpConfig, options = { timeoutMs: config.requestTimeoutMs }) {
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
			this.pending = this.pendingRequest(id, tool, resolveRequest, rejectRequest, signal);
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
	provenance(): McpSessionProvenance {
		const childProcessId = this.child?.pid;
		if (!childProcessId) throw new Error("KAMN live MCP session has no successful runtime provenance");
		return this.provenanceTracker.provenance(childProcessId);
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
	private pendingRequest(id: string, tool: string, resolveRequest: PendingRequest["resolve"], reject: PendingRequest["reject"], signal?: AbortSignal): PendingRequest {
		const timer = setTimeout(() => this.failSession(new Error(`KAMN live MCP request ${id} timed out`)), this.options.timeoutMs);
		const abort = () => this.failSession(new Error(`KAMN live MCP request ${id} aborted`));
		signal?.addEventListener("abort", abort, { once: true });
		return { id, tool, resolve: resolveRequest, reject, timer, removeAbort: () => signal?.removeEventListener("abort", abort) };
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
		if (response.ok !== true) {
			this.clearPending();
			this.provenanceTracker.record(pending.id, pending.tool, "error", isObject(response.error) ? response.error : {});
			return pending.reject(toolError(response));
		}
		if (!isObject(response.result)) return this.failSession(new Error("KAMN live MCP success response omitted result"));
		this.provenanceTracker.record(pending.id, pending.tool, "success", response.result);
		this.resolveSuccess(pending, response.result);
	}
	private resolveSuccess(pending: PendingRequest, result: JsonObject) {
		try {
			const authority = validateAuthority(pending.tool, result, this.registeredActor);
			if (authority.profileCommitment) {
				this.registeredActor = String(authority.serviceResult.did);
				this.provenanceTracker.recordProfileCommitment(authority.profileCommitment);
			}
			for (const receipt of authority.receipts ?? []) this.provenanceTracker.recordAuthorityReceipt(receipt);
			this.clearPending();
			pending.resolve(authority.serviceResult);
		} catch (error) {
			this.failSession(error instanceof Error ? error : new Error("MCP_AUTHORITY_RECEIPT_INVALID"));
		}
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
function isObject(value: unknown): value is JsonObject {
	return typeof value === "object" && value !== null && !Array.isArray(value);
}
function toolError(response: JsonObject): Error {
	const error = isObject(response.error) ? response.error : {};
	return new Error(`${String(error.kind ?? "unknown_error")}: ${String(error.message ?? "KAMN live MCP tool failed")}`);
}
