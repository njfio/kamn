import { createHash } from "node:crypto";

const SCHEMA = "kamn.mvp.live-task-handoff.v2";
const FIELDS = ["artifact_digest", "provider_did", "schema_version", "task_id", "terms_digest", "transaction_id"];
export type TaskHandoffInput = { task_id: string; transaction_id: string; terms_digest: string; provider_did: string };
export type TaskHandoff = TaskHandoffInput & { schema_version: string; artifact_digest: string };

export function buildTaskHandoff(input: TaskHandoffInput): TaskHandoff {
	const fields = validatedFields(input);
	const unsigned = { schema_version: SCHEMA, ...fields };
	return { ...unsigned, artifact_digest: digest(unsigned) };
}

export function parseTaskHandoff(raw: string): TaskHandoff {
	let parsed: unknown;
	try { parsed = JSON.parse(raw); } catch { throw new Error("KAMN live task coordination artifact is malformed JSON"); }
	if (!isRecord(parsed) || parsed.schema_version !== SCHEMA) throw new Error("KAMN live task coordination artifact schema mismatch");
	if (Object.keys(parsed).sort().join(",") !== FIELDS.join(",")) throw new Error("KAMN live task coordination artifact field mismatch");
	const fields = validatedFields(parsed as TaskHandoffInput);
	const unsigned = { schema_version: SCHEMA, ...fields };
	if (parsed.artifact_digest !== digest(unsigned)) throw new Error("KAMN live task coordination artifact digest mismatch");
	return { ...unsigned, artifact_digest: parsed.artifact_digest as string };
}

export function taskHandoffInput(handoff: TaskHandoff): TaskHandoffInput {
	return { task_id: handoff.task_id, transaction_id: handoff.transaction_id, terms_digest: handoff.terms_digest, provider_did: handoff.provider_did };
}

function validatedFields(input: TaskHandoffInput): TaskHandoffInput {
	if (!/^[A-Za-z0-9._:-]{1,200}$/.test(input.task_id)) throw new Error("KAMN live task ID is invalid");
	if (!/^[A-Za-z0-9._:-]{1,200}$/.test(input.transaction_id)) throw new Error("KAMN live transaction ID is invalid");
	if (!/^[0-9a-f]{64}$/.test(input.terms_digest)) throw new Error("KAMN live terms digest is invalid");
	if (!/^kamn:did:[A-Za-z0-9._:-]+$/.test(input.provider_did)) throw new Error("KAMN live provider DID is invalid");
	return {
		task_id: input.task_id,
		transaction_id: input.transaction_id,
		terms_digest: input.terms_digest,
		provider_did: input.provider_did,
	};
}

function digest(value: Record<string, unknown>): string {
	return createHash("sha256").update(JSON.stringify(value)).digest("hex");
}
function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === "object" && value !== null && !Array.isArray(value);
}
