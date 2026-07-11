import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import type { AgentRole, LiveTaskWorkflow, WorkflowResult } from "./live-task-workflow.ts";

type WorkflowResolver = (cwd: string) => LiveTaskWorkflow;

export function registerLiveTransactionTools(pi: ExtensionAPI, workflow: WorkflowResolver) {
	registerFund(pi, workflow);
	registerComplete(pi, workflow);
	registerRelease(pi, workflow);
	registerParticipantProjection(pi, workflow, "agent_a", "kamn_live_agent_a_query_participant_projection");
	registerParticipantProjection(pi, workflow, "agent_b", "kamn_live_agent_b_query_participant_projection");
	registerVerifierProjection(pi, workflow);
}

function registerFund(pi: ExtensionAPI, resolveWorkflow: WorkflowResolver) {
	registerTool(pi, {
		name: "kamn_live_agent_a_fund_escrow",
		label: "KAMN Agent A Fund Escrow",
		parameters: Type.Object({}),
		run: (workflow, _params, signal) => workflow.fundEscrow(signal),
		role: "agent_a",
	}, resolveWorkflow);
}

function registerComplete(pi: ExtensionAPI, resolveWorkflow: WorkflowResolver) {
	registerTool(pi, {
		name: "kamn_live_agent_b_complete_task",
		label: "KAMN Agent B Complete Task",
		parameters: Type.Object({}),
		run: (workflow, _params, signal) => workflow.completeTask(signal),
		role: "agent_b",
	}, resolveWorkflow);
}

function registerRelease(pi: ExtensionAPI, resolveWorkflow: WorkflowResolver) {
	registerTool(pi, {
		name: "kamn_live_agent_a_release_escrow",
		label: "KAMN Agent A Release Escrow",
		parameters: Type.Object({}),
		run: (workflow, _params, signal) => workflow.releaseEscrow(signal),
		role: "agent_a",
	}, resolveWorkflow);
}

function registerParticipantProjection(
	pi: ExtensionAPI,
	resolveWorkflow: WorkflowResolver,
	role: "agent_a" | "agent_b",
	name: string,
) {
	const actor = role === "agent_a" ? "a" : "b";
	registerTool(pi, {
		name,
		label: `KAMN Agent ${actor.toUpperCase()} Participant Projection`,
		parameters: Type.Object({}),
		run: (workflow, _params, signal) => workflow.queryParticipantProjection(role, signal),
		role,
	}, resolveWorkflow);
}

function registerVerifierProjection(pi: ExtensionAPI, resolveWorkflow: WorkflowResolver) {
	registerTool(pi, {
		name: "kamn_live_agent_c_query_verifier_projection",
		label: "KAMN Agent C Verifier Projection",
		parameters: Type.Object({}),
		run: (workflow, _params, signal) => workflow.queryVerifierProjection(signal),
		role: "agent_c",
	}, resolveWorkflow);
}

type ToolSpec = {
	name: string;
	label: string;
	parameters: ReturnType<typeof Type.Object>;
	role: AgentRole;
	run: (workflow: LiveTaskWorkflow, params: Record<string, unknown>, signal?: AbortSignal) => Promise<WorkflowResult>;
};

function registerTool(pi: ExtensionAPI, spec: ToolSpec, resolveWorkflow: WorkflowResolver) {
	pi.registerTool({
		name: spec.name,
		label: spec.label,
		description: `${spec.label} through the role's persistent authenticated MCP session.`,
		parameters: spec.parameters,
		executionMode: "sequential",
		async execute(_id, params, signal, _onUpdate, ctx) {
			const workflow = resolveWorkflow(ctx.cwd);
			const result = await spec.run(workflow, params as Record<string, unknown>, signal);
			return transactionResult(spec.label, result, workflow.provenance(spec.role));
		},
	});
}

function transactionResult(label: string, result: WorkflowResult, provenance: unknown) {
	const claimBoundary = "authenticated runtime operation; settlement requires canonical devnet verification";
	return {
		content: [{ type: "text" as const, text: `${label} passed. Claim boundary: ${claimBoundary}.` }],
		details: { claimBoundary, result, provenance },
	};
}
