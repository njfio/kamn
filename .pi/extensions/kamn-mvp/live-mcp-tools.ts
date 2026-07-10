import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import { McpSession, readLiveMcpConfig } from "./mcp-session.ts";

type LiveAgentState = { session?: McpSession; did?: string };

export function registerLiveMcpTools(pi: ExtensionAPI) {
	const state: LiveAgentState = {};
	pi.registerTool({
		name: "kamn_live_agent_a_register",
		label: "KAMN Live Agent A Register",
		description: "Register Agent A through a persistent live local KAMN MCP process.",
		promptSnippet: "Register Agent A through the live local-only KAMN service path",
		parameters: Type.Object({}),
		executionMode: "sequential",
		async execute(_id, _params, signal, _onUpdate, ctx) {
			const result = await session(state, ctx.cwd).call("register", {}, signal);
			state.did = requiredDid(result);
			return textResult("Agent A registration persisted through the local KAMN service.", result);
		},
	});
	pi.registerTool({
		name: "kamn_live_agent_a_query_profile",
		label: "KAMN Live Agent A Query Profile",
		description: "Query Agent A's durable profile through the same live local KAMN MCP process.",
		promptSnippet: "Query the registered Agent A profile through the live local-only KAMN service path",
		parameters: Type.Object({}),
		executionMode: "sequential",
		async execute(_id, _params, signal, _onUpdate, ctx) {
			if (!state.did) throw new Error("Register Agent A before querying its live profile");
			const result = await session(state, ctx.cwd).call("query_agent_profile", { did: state.did }, signal);
			return textResult("Agent A durable profile query passed through the same local MCP process.", result);
		},
	});
	pi.on("session_shutdown", async () => state.session?.shutdown());
}

function session(state: LiveAgentState, cwd: string): McpSession {
	state.session ??= new McpSession(readLiveMcpConfig(process.env, cwd));
	return state.session;
}

function requiredDid(result: Record<string, unknown>): string {
	if (typeof result.did !== "string" || !result.did) throw new Error("KAMN live registration omitted Agent A DID");
	return result.did;
}

function textResult(text: string, result: Record<string, unknown>) {
	return {
		content: [{ type: "text" as const, text: `${text} Claim boundary: local-only identity durability; no settlement or asset movement is claimed.` }],
		details: { claimBoundary: "local-only identity durability", result },
	};
}
