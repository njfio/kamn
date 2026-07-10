#!/usr/bin/env node
import { appendFileSync, writeFileSync } from "node:fs";
import { createInterface } from "node:readline";

const mode = process.env.KAMN_MVP_FAKE_MCP_MODE ?? "success";
const startFile = process.env.KAMN_MVP_FAKE_MCP_START_FILE;
const stopFile = process.env.KAMN_MVP_FAKE_MCP_STOP_FILE;
if (startFile) writeFileSync(startFile, `${process.pid}\n`);

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
	const did = request.did ?? "kamn:did:agent-a";
	return {
		ok: true,
		id: mode === "mismatch" ? "wrong-id" : request.id,
		tool: request.tool,
		result: { did, pid: process.pid, request_id: request.id },
	};
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
