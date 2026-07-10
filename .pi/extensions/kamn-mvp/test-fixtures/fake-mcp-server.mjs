#!/usr/bin/env node
import { appendFileSync } from "node:fs";
import { createInterface } from "node:readline";

const mode = process.env.KAMN_MVP_FAKE_MCP_MODE ?? "success";
const startFile = process.env.KAMN_MVP_FAKE_MCP_START_FILE;
const stopFile = process.env.KAMN_MVP_FAKE_MCP_STOP_FILE;
const agentName = argumentValue("--agent-name") ?? "agent-a";
if (startFile) appendFileSync(startFile, `${process.pid}\n`);

process.on("SIGTERM", () => {
	if (stopFile) appendFileSync(stopFile, `${process.pid}\n`);
	process.exit(0);
});

createInterface({ input: process.stdin }).on("line", (line) => {
	if (mode === "exit") process.exit(7);
	if (mode === "hang") return;
	if (mode === "malformed") return process.stdout.write("not-json\n");
	const request = JSON.parse(line);
	if (mode === "error") return write(errorResponse(request));
	write(successResponse(request));
});

function successResponse(request) {
	const result = toolResult(request);
	return {
		ok: true,
		id: mode === "mismatch" ? "wrong-id" : request.id,
		tool: request.tool,
		result: { ...result, pid: process.pid, request_id: request.id },
	};
}

function toolResult(request) {
	if (request.tool === "register") return { did: `kamn:did:${agentName}` };
	if (request.tool === "query_agent_profile") return { did: request.did };
	if (request.tool === "create_task") {
		const payload = JSON.parse(request.payload);
		return { task_id: "task-live-1", state: "submitted", ...payload };
	}
	if (["accept_task", "query_task"].includes(request.tool)) {
		return { task_id: request.task_id, state: "accepted" };
	}
	return {};
}

function errorResponse(request) {
	return {
		ok: false,
		id: request.id,
		tool: request.tool,
		error: { kind: "backend_error", message: "forced backend failure" },
	};
}

function write(response) {
	process.stdout.write(`${JSON.stringify(response)}\n`);
}

function argumentValue(flag) {
	const index = process.argv.indexOf(flag);
	return index >= 0 ? process.argv[index + 1] : undefined;
}
