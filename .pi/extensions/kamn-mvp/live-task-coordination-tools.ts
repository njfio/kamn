import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import {
	COORDINATION_TOOL_NAMES,
	coordinationConfig,
	verifyIndependentActorReceipts,
	waitForTaskHandoff,
	writeActorReceipt,
	writeTaskHandoff,
} from "./live-task-coordination.ts";
import type { LiveTaskWorkflow, WorkflowResult } from "./live-task-workflow.ts";

type WorkflowResolver = (cwd: string) => LiveTaskWorkflow;

export function registerLiveTaskCoordinationTools(pi: ExtensionAPI, resolveWorkflow: WorkflowResolver) {
	registerPublishTool(pi, resolveWorkflow);
	registerReceiveTool(pi, resolveWorkflow);
	registerAgentAWaitTool(pi, resolveWorkflow);
	registerAgentBReceiptTool(pi, resolveWorkflow);
	registerVerifierTool(pi);
}

function registerPublishTool(pi: ExtensionAPI, resolveWorkflow: WorkflowResolver) {
	pi.registerTool({
		name: COORDINATION_TOOL_NAMES[0],
		label: "KAMN Agent A Publish Task Handoff",
		description: "Publish the current non-secret task ID for an independent Agent B Pi process.",
		parameters: Type.Object({}),
		executionMode: "sequential",
		async execute(_id, _params, _signal, _onUpdate, ctx) {
			const config = coordinationConfig(process.env, ctx.cwd);
			const taskId = resolveWorkflow(ctx.cwd).currentTaskId();
			await writeTaskHandoff(config.handoffPath, taskId);
			return coordinationResult("Agent A published the task handoff.", { task_id: taskId });
		},
	});
}

function registerReceiveTool(pi: ExtensionAPI, resolveWorkflow: WorkflowResolver) {
	pi.registerTool({
		name: COORDINATION_TOOL_NAMES[1],
		label: "KAMN Agent B Receive Task Handoff",
		description: "Wait for and validate Agent A's task handoff in an independent Pi process.",
		parameters: Type.Object({}),
		executionMode: "sequential",
		async execute(_id, _params, signal, _onUpdate, ctx) {
			const config = coordinationConfig(process.env, ctx.cwd);
			const taskId = await waitForTaskHandoff(config.handoffPath, config.options, signal);
			resolveWorkflow(ctx.cwd).importTask(taskId);
			return coordinationResult("Agent B received the task handoff.", { task_id: taskId });
		},
	});
}

function registerAgentAWaitTool(pi: ExtensionAPI, resolveWorkflow: WorkflowResolver) {
	pi.registerTool({
		name: COORDINATION_TOOL_NAMES[2],
		label: "KAMN Agent A Wait For Task Acceptance",
		description: "Poll through Agent A's persistent MCP session and record accepted state.",
		parameters: Type.Object({}),
		executionMode: "sequential",
		async execute(_id, _params, signal, _onUpdate, ctx) {
			const config = coordinationConfig(process.env, ctx.cwd);
			const result = await resolveWorkflow(ctx.cwd).waitForAccepted("agent_a", config.options, signal);
			await writeObservation(config.agentAReceiptPath, "agent_a", result);
			return coordinationResult("Agent A independently observed accepted state.", result);
		},
	});
}

function registerAgentBReceiptTool(pi: ExtensionAPI, resolveWorkflow: WorkflowResolver) {
	pi.registerTool({
		name: COORDINATION_TOOL_NAMES[3],
		label: "KAMN Agent B Write Task Receipt",
		description: "Write Agent B's accepted-state receipt from its independent Pi process.",
		parameters: Type.Object({}),
		executionMode: "sequential",
		async execute(_id, _params, _signal, _onUpdate, ctx) {
			const config = coordinationConfig(process.env, ctx.cwd);
			const result = resolveWorkflow(ctx.cwd).acceptedObservation("agent_b");
			await writeObservation(config.agentBReceiptPath, "agent_b", result);
			return coordinationResult("Agent B wrote its independent accepted-state receipt.", result);
		},
	});
}

function registerVerifierTool(pi: ExtensionAPI) {
	pi.registerTool({
		name: COORDINATION_TOOL_NAMES[4],
		label: "KAMN Verify Independent Actor Receipts",
		description: "Verify Agent A and Agent B receipts came from distinct Pi processes and agree.",
		parameters: Type.Object({}),
		executionMode: "sequential",
		async execute(_id, _params, _signal, _onUpdate, ctx) {
			const config = coordinationConfig(process.env, ctx.cwd);
			const result = await verifyIndependentActorReceipts(config.handoffPath, config.agentAReceiptPath, config.agentBReceiptPath);
			return coordinationResult("Independent Pi actor receipts verified.", result);
		},
	});
}

async function writeObservation(path: string, actor: "agent_a" | "agent_b", result: WorkflowResult) {
	const taskId = requiredString(result, "task_id");
	const state = requiredString(result, "state");
	await writeActorReceipt(path, actor, taskId, state, process.pid);
}

function requiredString(result: WorkflowResult, field: string): string {
	const value = result[field];
	if (typeof value !== "string" || !value) throw new Error(`KAMN live task observation omitted ${field}`);
	return value;
}

function coordinationResult(text: string, result: Record<string, unknown>) {
	return {
		content: [{ type: "text" as const, text: `${text} Claim boundary: real local-only independent Pi actors.` }],
		details: { claimBoundary: "real local-only independent Pi actors", result },
	};
}
