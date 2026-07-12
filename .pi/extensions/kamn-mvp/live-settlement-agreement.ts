import { createHash } from "node:crypto";

type Environment = Record<string, string | undefined>;
export type AgreementIdentity = { transaction_id: string; terms_digest: string; provider_did: string };

export class LiveSettlementAgreement {
	readonly transactionId: string;
	readonly termsDigest: string;
	readonly amountLamports: number;
	private readonly creatorDid: string;
	private readonly providerDid: string;
	private readonly completionDigest: string;
	constructor(creatorDid: string, providerDid: string, env: Environment) {
		this.creatorDid = creatorDid;
		this.providerDid = providerDid;
		this.amountLamports = requiredLamports(env.KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS);
		const runId = requiredValue(env.KAMN_MVP_PI_RUN_ID, "KAMN_MVP_PI_RUN_ID");
		const digest = sha256(`${runId}:${creatorDid}:${providerDid}:${this.amountLamports}`);
		this.transactionId = `pi-devnet-${digest.slice(0, 16)}`;
		this.termsDigest = digest;
		this.completionDigest = sha256(`completed:${digest}`);
	}
	taskPayload(title: string, description: string): string {
		return JSON.stringify({
			title, description, provider_did: this.providerDid,
			transaction_id: this.transactionId, terms_digest: this.termsDigest,
			idempotency_key: this.operationKey("create"),
		});
	}
	fundPayload(taskId: string): string {
		return JSON.stringify({
			task_id: taskId, transaction_id: this.transactionId,
			beneficiary_did: this.providerDid, amount_lamports: this.amountLamports,
			network: "solana-devnet", terms_digest: this.termsDigest,
			release_authority_did: this.creatorDid, release_policy: "task-completed",
			idempotency_key: this.operationKey("fund"),
		});
	}
	releasePayload(): string {
		return JSON.stringify({ idempotency_key: this.operationKey("release") });
	}
	identity(): AgreementIdentity {
		return { transaction_id: this.transactionId, terms_digest: this.termsDigest, provider_did: this.providerDid };
	}
	static taskOperationPayload(identity: AgreementIdentity, operation: "accept" | "complete"): string {
		return JSON.stringify({
			idempotency_key: `${identity.transaction_id}-${operation}`,
			...(operation === "complete" ? { completion_evidence_digest: sha256(`completed:${identity.terms_digest}`) } : {}),
		});
	}
	private operationKey(operation: string): string {
		return `${this.transactionId}-${operation}`;
	}
}

function requiredLamports(value: string | undefined): number {
	const parsed = Number(value);
	if (!Number.isSafeInteger(parsed) || parsed <= 0) throw new Error("KAMN live settlement lamports must be a positive integer");
	return parsed;
}

function requiredValue(value: string | undefined, name: string): string {
	if (!value?.trim()) throw new Error(`${name} must not be blank`);
	return value.trim();
}

function sha256(value: string): string {
	return createHash("sha256").update(value).digest("hex");
}
