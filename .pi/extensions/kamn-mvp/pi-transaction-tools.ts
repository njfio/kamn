import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import { resolve } from "node:path";
import type { AgentRole, LiveTaskWorkflow } from "./live-task-workflow.ts";
import { readTaskHandoffEvidence } from "./live-task-coordination.ts";
import { verifyPiTransactionActors, writePiTransactionActor } from "./pi-transaction-evidence.ts";

type WorkflowResolver = (cwd: string) => LiveTaskWorkflow;
type Role = "agent_a" | "agent_b" | "agent_c";
type Paths = Record<Role, string> & { handoff: string };

export function registerPiTransactionEvidenceTools(pi: ExtensionAPI, workflow: WorkflowResolver) {
	registerWriter(pi, workflow, "agent_a", "kamn_live_agent_a_write_transaction_evidence");
	registerWriter(pi, workflow, "agent_b", "kamn_live_agent_b_write_transaction_evidence");
	registerWriter(pi, workflow, "agent_c", "kamn_live_agent_c_write_transaction_evidence");
	registerVerifier(pi);
}

function registerWriter(pi: ExtensionAPI, resolveWorkflow: WorkflowResolver, role: Role, name: string) {
	pi.registerTool({
		name,
		label: `${role.replace("_", " ")} transaction evidence`,
		description: "Persist this role's server-derived transaction evidence and MCP provenance.",
		parameters: Type.Object({}),
		executionMode: "sequential",
		async execute(_id, _params, _signal, _onUpdate, ctx) {
			const paths = evidencePaths(process.env, ctx.cwd);
			const handoff = await readTaskHandoffEvidence(paths.handoff);
			const workflow = resolveWorkflow(ctx.cwd);
			if (workflow.currentTaskId() !== handoff.task_id) throw new Error("PI_TRANSACTION_FACT_MISMATCH");
			const input = workflow.actorEvidence(role as AgentRole, process.pid, handoff.artifact_digest);
			await writePiTransactionActor(paths[role], input);
			return resultEnvelope(`${role} transaction evidence persisted.`, { actor: role, path: paths[role] });
		},
	});
}

function registerVerifier(pi: ExtensionAPI) {
	pi.registerTool({
		name: "kamn_live_verify_pi_transaction_actors",
		label: "KAMN Verify Pi Transaction Actors",
		description: "Verify three independent Pi actor artifacts against one transaction.",
		parameters: Type.Object({}),
		executionMode: "sequential",
		async execute(_id, _params, _signal, _onUpdate, ctx) {
			const paths = evidencePaths(process.env, ctx.cwd);
			const verified = await verifyPiTransactionActors(paths);
			return resultEnvelope("Independent Pi transaction actors verified.", verified);
		},
	});
}

function evidencePaths(env: Record<string, string | undefined>, cwd: string): Paths {
	return {
		handoff: configuredPath(env, "KAMN_MVP_LIVE_TASK_HANDOFF_FILE", cwd),
		agent_a: configuredPath(env, "KAMN_MVP_PI_TRANSACTION_AGENT_A_FILE", cwd),
		agent_b: configuredPath(env, "KAMN_MVP_PI_TRANSACTION_AGENT_B_FILE", cwd),
		agent_c: configuredPath(env, "KAMN_MVP_PI_TRANSACTION_AGENT_C_FILE", cwd),
	};
}

function configuredPath(env: Record<string, string | undefined>, name: string, cwd: string): string {
	const value = env[name]?.trim();
	if (!value) throw new Error(`Missing required environment variable: ${name}`);
	const path = value.startsWith("/") ? value : resolve(cwd, value);
	if (/[.]kamn\/devnet|auth[.]json|[.]env|keypair|id_rsa|oauth/i.test(path)) throw new Error("Refusing secret-like Pi transaction evidence path");
	return path;
}

function resultEnvelope(text: string, result: unknown) {
	const claimBoundary = "runtime-bound independent Pi actors; settlement requires canonical devnet verification";
	return {
		content: [{ type: "text" as const, text: `${text} Claim boundary: ${claimBoundary}.` }],
		details: { claimBoundary, result },
	};
}
