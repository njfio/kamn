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
import {
	restrictedObservationConfig,
	writeRestrictedTaskObservation,
} from "./restricted-task-observation.ts";

type WorkflowResolver = (cwd: string) => LiveTaskWorkflow;

export function registerLiveTaskCoordinationTools(pi: ExtensionAPI, resolveWorkflow: WorkflowResolver) {
	registerPublishTool(pi, resolveWorkflow);
	registerReceiveTool(pi, resolveWorkflow);
	registerAgentCReceiveTool(pi, resolveWorkflow);
	registerAgentAWaitTool(pi, resolveWorkflow);
	registerAgentBReceiptTool(pi, resolveWorkflow);
	registerVerifierTool(pi);
	registerAgentCObservationTool(pi);
}

function registerAgentCReceiveTool(pi: ExtensionAPI, resolveWorkflow: WorkflowResolver) {
	pi.registerTool({
		name: "kamn_live_agent_c_receive_task_handoff",
		label: "KAMN Agent C Receive Task Handoff",
		description: "Receive the non-secret task ID for Agent C without granting authorization.",
		parameters: Type.Object({}),
		executionMode: "sequential",
		async execute(_id, _params, signal, _onUpdate, ctx) {
			const config = coordinationConfig(process.env, ctx.cwd);
			const handoff = await waitForTaskHandoff(config.handoffPath, config.options, signal);
			resolveWorkflow(ctx.cwd).importTask(handoff.task_id);
			return coordinationResult("Agent C received the identifier-only task handoff.", { task_id: handoff.task_id });
		},
	});
}

function registerAgentCObservationTool(pi: ExtensionAPI) {
	pi.registerTool({
		name: "kamn_live_agent_c_verify_restricted_task_observation",
		label: "KAMN Agent C Verify Restricted Task Observation",
		description: "Verify a restricted task observation in a third independent Pi process.",
		parameters: Type.Object({}),
		executionMode: "sequential",
		async execute(_id, _params, _signal, _onUpdate, ctx) {
			const config = restrictedObservationConfig(process.env, ctx.cwd);
			const result = await writeRestrictedTaskObservation(
				config.handoffPath, config.agentAPath, config.agentBPath, config.observationPath, process.pid,
			);
			return {
				content: [{ type: "text" as const, text: `Agent C verified the restricted task observation. Claim boundary: ${result.claim_boundary}.` }],
				details: { claimBoundary: result.claim_boundary, result },
			};
		},
	});
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
			const handoff = resolveWorkflow(ctx.cwd).taskHandoff();
			await writeTaskHandoff(config.handoffPath, handoff);
			return coordinationResult("Agent A published the task handoff.", { task_id: handoff.task_id });
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
			const handoff = await waitForTaskHandoff(config.handoffPath, config.options, signal);
			resolveWorkflow(ctx.cwd).importTask(handoff.task_id, handoff);
			return coordinationResult("Agent B received the task handoff.", { task_id: handoff.task_id });
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
