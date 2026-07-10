import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import { LiveTaskWorkflow, type AgentRole, type WorkflowResult } from "./live-task-workflow.ts";

type WorkflowHolder = { workflow?: LiveTaskWorkflow };

export function registerLiveMcpTools(pi: ExtensionAPI) {
	const holder: WorkflowHolder = {};
	registerAgent(pi, holder, "agent_a", "kamn_live_agent_a_register", "KAMN Live Agent A Register");
	registerAgent(pi, holder, "agent_b", "kamn_live_agent_b_register", "KAMN Live Agent B Register");
	registerLiveAgentAQuery(pi, holder);
	registerTaskTools(pi, holder);
	pi.on("session_shutdown", async () => holder.workflow?.shutdown());
}

function registerAgent(pi: ExtensionAPI, holder: WorkflowHolder, role: AgentRole, name: string, label: string) {
	const agent = role === "agent_a" ? "Agent A" : "Agent B";
	pi.registerTool({
		name,
		label,
		description: `Register ${agent} through its persistent live local KAMN MCP process.`,
		promptSnippet: `Register ${agent} through the live local-only KAMN service path`,
		parameters: Type.Object({}),
		executionMode: "sequential",
		async execute(_id, _params, signal, _onUpdate, ctx) {
			const result = await workflow(holder, ctx.cwd).register(role, signal);
			return identityResult(`${agent} registration persisted through the local KAMN service.`, result);
		},
	});
}

function registerLiveAgentAQuery(pi: ExtensionAPI, holder: WorkflowHolder) {
	pi.registerTool({
		name: "kamn_live_agent_a_query_profile",
		label: "KAMN Live Agent A Query Profile",
		description: "Query Agent A's durable profile through the same live local KAMN MCP process.",
		promptSnippet: "Query the registered Agent A profile through the live local-only KAMN service path",
		parameters: Type.Object({}),
		executionMode: "sequential",
		async execute(_id, _params, signal, _onUpdate, ctx) {
			const result = await workflow(holder, ctx.cwd).queryProfile("agent_a", signal);
			return identityResult("Agent A durable profile query passed through the same local MCP process.", result);
		},
	});
}

function registerTaskTools(pi: ExtensionAPI, holder: WorkflowHolder) {
	registerCreateTask(pi, holder);
	registerAcceptTask(pi, holder);
	registerQueryTask(pi, holder, "agent_a", "kamn_live_agent_a_query_task", "KAMN Live Agent A Query Task");
	registerQueryTask(pi, holder, "agent_b", "kamn_live_agent_b_query_task", "KAMN Live Agent B Query Task");
}

function registerCreateTask(pi: ExtensionAPI, holder: WorkflowHolder) {
	pi.registerTool({
		name: "kamn_live_agent_a_create_task",
		label: "KAMN Live Agent A Create Task",
		description: "Create one durable local KAMN task through Agent A's live MCP process.",
		promptSnippet: "Create a real local-only KAMN task as Agent A",
		parameters: Type.Object({ title: Type.String({ minLength: 1 }), description: Type.String({ minLength: 1 }) }),
		executionMode: "sequential",
		async execute(_id, params, signal, _onUpdate, ctx) {
			const result = await workflow(holder, ctx.cwd).createTask(params.title, params.description, signal);
			return taskResult("Agent A created a durable local KAMN task.", result);
		},
	});
}

function registerAcceptTask(pi: ExtensionAPI, holder: WorkflowHolder) {
	pi.registerTool({
		name: "kamn_live_agent_b_accept_task",
		label: "KAMN Live Agent B Accept Task",
		description: "Accept Agent A's durable task through Agent B's independent live MCP process.",
		promptSnippet: "Accept the current real local-only KAMN task as Agent B",
		parameters: Type.Object({}),
		executionMode: "sequential",
		async execute(_id, _params, signal, _onUpdate, ctx) {
			const result = await workflow(holder, ctx.cwd).acceptTask(signal);
			return taskResult("Agent B accepted Agent A's durable local KAMN task.", result);
		},
	});
}

function registerQueryTask(pi: ExtensionAPI, holder: WorkflowHolder, role: AgentRole, name: string, label: string) {
	pi.registerTool({
		name,
		label,
		description: "Query the accepted durable task through one participant's independent live MCP process.",
		promptSnippet: `Query the current accepted KAMN task as ${role === "agent_a" ? "Agent A" : "Agent B"}`,
		parameters: Type.Object({}),
		executionMode: "sequential",
		async execute(_id, _params, signal, _onUpdate, ctx) {
			const result = await workflow(holder, ctx.cwd).queryTask(role, signal);
			return taskResult(`${label} observed the accepted durable task.`, result);
		},
	});
}

function workflow(holder: WorkflowHolder, cwd: string): LiveTaskWorkflow {
	holder.workflow ??= new LiveTaskWorkflow(process.env, cwd);
	return holder.workflow;
}

function identityResult(text: string, result: WorkflowResult) {
	return resultEnvelope(text, "local-only identity durability", result);
}

function taskResult(text: string, result: WorkflowResult) {
	return resultEnvelope(text, "real local-only task lifecycle", result);
}

function resultEnvelope(text: string, claimBoundary: string, result: WorkflowResult) {
	return {
		content: [{ type: "text" as const, text: `${text} Claim boundary: ${claimBoundary}; no escrow, settlement, or asset movement is claimed.` }],
		details: { claimBoundary, result },
	};
}
