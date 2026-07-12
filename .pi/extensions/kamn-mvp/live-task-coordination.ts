import { createHash } from "node:crypto";
import { readFile, stat, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import type { VerifiedActorEvidence } from "./live-task-evidence.ts";
import { buildTaskHandoff, parseTaskHandoff, taskHandoffInput, type TaskHandoffInput } from "./live-task-handoff.ts";

const RECEIPT_SCHEMA = "kamn.mvp.live-task-actor-receipt.v1";
const SECRET_MARKERS = [".kamn/devnet", "auth.json", ".env", "keypair", "id_rsa", "oauth"];
export const COORDINATION_TOOL_NAMES = [
	"kamn_live_agent_a_publish_task_handoff",
	"kamn_live_agent_b_receive_task_handoff",
	"kamn_live_agent_a_wait_for_task_acceptance",
	"kamn_live_agent_b_write_task_receipt",
	"kamn_live_verify_independent_actor_receipts",
] as const;
export type CoordinationOptions = { timeoutMs: number; pollMs: number; maxAgeMs: number };
type Environment = Record<string, string | undefined>;
type Actor = "agent_a" | "agent_b";
type Handoff = ReturnType<typeof parseTaskHandoff>;
type Receipt = { schema_version: string; task_id: string; artifact_digest: string; actor: Actor; state: string; pi_process_id: number };

export async function writeTaskHandoff(path: string, input: TaskHandoffInput): Promise<void> {
	assertSafePath(path);
	const artifact = buildTaskHandoff(input);
	await writeIdempotent(path, artifact, "task handoff");
}
export async function waitForTaskHandoff(path: string, options: CoordinationOptions, signal?: AbortSignal): Promise<TaskHandoffInput> {
	assertSafePath(path);
	validateOptions(options);
	const deadline = Date.now() + options.timeoutMs;
	while (Date.now() <= deadline) {
		if (signal?.aborted) throw new Error("KAMN live task handoff wait aborted");
		const handoff = await readHandoffIfPresent(path, options.maxAgeMs);
		if (handoff) return taskHandoffInput(handoff);
		await delay(options.pollMs, signal);
	}
	throw new Error(`KAMN live task handoff timed out: ${path}`);
}
export async function readTaskHandoffEvidence(path: string) {
	const handoff = await readHandoff(path);
	return { ...taskHandoffInput(handoff), artifact_digest: `sha256:${handoff.artifact_digest}` };
}
export async function writeActorReceipt(path: string, actor: Actor, taskId: string, state: string, piProcessId: number): Promise<void> {
	assertSafePath(path);
	if (state !== "accepted") throw new Error("KAMN live task actor receipt state must be accepted");
	if (!Number.isInteger(piProcessId) || piProcessId <= 0) throw new Error("KAMN live task actor receipt Pi process ID must be positive");
	const artifact = withDigest({
		schema_version: RECEIPT_SCHEMA,
		actor,
		task_id: validTaskId(taskId),
		state,
		pi_process_id: piProcessId,
	});
	await writeIdempotent(path, artifact, `${actor} receipt`);
}

export async function verifyIndependentActorReceipts(handoffPath: string, agentAPath: string, agentBPath: string) {
	const evidence = await readVerifiedActorEvidence(handoffPath, agentAPath, agentBPath);
	return {
		claim_boundary: "real local-only independent Pi actors",
		task_id: evidence.task_id,
		state: evidence.state,
		agent_a_pi_process_id: evidence.agent_a_pi_process_id,
		agent_b_pi_process_id: evidence.agent_b_pi_process_id,
	};
}

export async function readVerifiedActorEvidence(
	handoffPath: string,
	agentAPath: string,
	agentBPath: string,
): Promise<VerifiedActorEvidence> {
	const handoff = await readHandoff(handoffPath);
	const agentA = await readReceipt(agentAPath, "agent_a");
	const agentB = await readReceipt(agentBPath, "agent_b");
	if (agentA.task_id !== handoff.task_id || agentB.task_id !== handoff.task_id) throw new Error("KAMN live actor receipt task ID mismatch");
	if (agentA.pi_process_id === agentB.pi_process_id) throw new Error("KAMN live actor receipts must come from distinct Pi processes");
	return {
		task_id: handoff.task_id,
		state: "accepted",
		agent_a_pi_process_id: agentA.pi_process_id,
		agent_b_pi_process_id: agentB.pi_process_id,
		source_handoff_digest: handoff.artifact_digest,
		source_agent_a_receipt_digest: agentA.artifact_digest,
		source_agent_b_receipt_digest: agentB.artifact_digest,
	};
}

export function coordinationConfig(env: Environment, cwd: string) {
	const timeoutMs = optionalPositiveInteger(env.KAMN_MVP_LIVE_TASK_COORDINATION_TIMEOUT_MS, 30000);
	return {
		handoffPath: configuredPath(env, "KAMN_MVP_LIVE_TASK_HANDOFF_FILE", cwd),
		agentAReceiptPath: configuredPath(env, "KAMN_MVP_LIVE_TASK_AGENT_A_RECEIPT_FILE", cwd),
		agentBReceiptPath: configuredPath(env, "KAMN_MVP_LIVE_TASK_AGENT_B_RECEIPT_FILE", cwd),
		options: { timeoutMs, pollMs: 100, maxAgeMs: 300000 },
	};
}

async function readHandoffIfPresent(path: string, maxAgeMs: number): Promise<Handoff | undefined> {
	try {
		const metadata = await stat(path);
		if (Date.now() - metadata.mtimeMs > maxAgeMs) throw new Error(`KAMN live task handoff is stale: ${path}`);
		return readHandoff(path);
	} catch (error) {
		if (isNodeError(error, "ENOENT")) return undefined;
		throw error;
	}
}

async function readHandoff(path: string): Promise<Handoff> {
	assertSafePath(path);
	return parseTaskHandoff(await readFile(path, "utf8"));
}

async function readReceipt(path: string, actor: Actor): Promise<Receipt> {
	assertSafePath(path);
	const artifact = parseArtifact(await readFile(path, "utf8"), RECEIPT_SCHEMA, ["actor", "artifact_digest", "pi_process_id", "schema_version", "state", "task_id"]) as Receipt;
	if (artifact.actor !== actor) throw new Error(`KAMN live task actor receipt expected ${actor}`);
	if (artifact.state !== "accepted") throw new Error("KAMN live task actor receipt state mismatch");
	if (!Number.isInteger(artifact.pi_process_id) || artifact.pi_process_id <= 0) throw new Error("KAMN live task actor receipt Pi process ID is invalid");
	validTaskId(artifact.task_id);
	return artifact;
}

function parseArtifact(raw: string, schema: string, keys: string[]): Record<string, unknown> {
	let artifact: Record<string, unknown>;
	try { artifact = JSON.parse(raw) as Record<string, unknown>; } catch { throw new Error("KAMN live task coordination artifact is malformed JSON"); }
	if (!artifact || Array.isArray(artifact) || artifact.schema_version !== schema) throw new Error("KAMN live task coordination artifact schema mismatch");
	if (Object.keys(artifact).sort().join(",") !== keys.join(",")) throw new Error("KAMN live task coordination artifact field mismatch");
	const digest = artifact.artifact_digest;
	const unsigned = Object.fromEntries(Object.entries(artifact).filter(([key]) => key !== "artifact_digest"));
	if (digest !== artifactDigest(unsigned)) throw new Error("KAMN live task coordination artifact digest mismatch");
	return artifact;
}

function withDigest<T extends Record<string, unknown>>(artifact: T): T & { artifact_digest: string } {
	return { ...artifact, artifact_digest: artifactDigest(artifact) };
}

function artifactDigest(artifact: Record<string, unknown>): string {
	return createHash("sha256").update(JSON.stringify(artifact)).digest("hex");
}

async function writeIdempotent(path: string, artifact: Record<string, unknown>, label: string) {
	const json = `${JSON.stringify(artifact)}\n`;
	try { await writeFile(path, json, { flag: "wx", mode: 0o600 }); } catch (error) {
		if (!isNodeError(error, "EEXIST")) throw error;
		if (await readFile(path, "utf8") !== json) throw new Error(`KAMN live ${label} conflicts with existing artifact`);
	}
}

function validTaskId(taskId: string): string {
	if (!/^[A-Za-z0-9._:-]{1,200}$/.test(taskId)) throw new Error("KAMN live task ID is invalid");
	return taskId;
}

function configuredPath(env: Environment, name: string, cwd: string): string {
	const value = env[name]?.trim();
	if (!value) throw new Error(`Missing required environment variable: ${name}`);
	const path = value.startsWith("/") ? value : resolve(cwd, value);
	assertSafePath(path);
	return path;
}

function assertSafePath(path: string) {
	const lower = path.toLowerCase();
	if (SECRET_MARKERS.some((marker) => lower.includes(marker))) throw new Error("Refusing secret-like KAMN live task coordination path");
}

function validateOptions(options: CoordinationOptions) {
	for (const [name, value] of Object.entries(options)) {
		if (!Number.isInteger(value) || value <= 0) throw new Error(`KAMN live task coordination ${name} must be positive`);
	}
}

function optionalPositiveInteger(raw: string | undefined, fallback: number): number {
	if (raw === undefined) return fallback;
	const value = Number(raw);
	if (!Number.isInteger(value) || value <= 0) throw new Error("KAMN live task coordination timeout must be positive");
	return value;
}

function delay(milliseconds: number, signal?: AbortSignal): Promise<void> {
	return new Promise((resolveDelay, reject) => {
		const finish = () => { signal?.removeEventListener("abort", abort); resolveDelay(); };
		const timer = setTimeout(finish, milliseconds);
		const abort = () => {
			clearTimeout(timer);
			signal?.removeEventListener("abort", abort);
			reject(new Error("KAMN live task handoff wait aborted"));
		};
		signal?.addEventListener("abort", abort, { once: true });
	});
}

function isNodeError(error: unknown, code: string): boolean {
	return error instanceof Error && "code" in error && error.code === code;
}
