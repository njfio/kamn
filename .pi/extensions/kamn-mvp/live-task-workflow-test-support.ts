import { createHash } from "node:crypto";
import { chmod, mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";

const fixture = resolve(".pi/extensions/kamn-mvp/test-fixtures/fake-mcp-server.mjs");

export async function testSetup(extra: Record<string, string> = {}) {
	const root = await mkdtemp(resolve(tmpdir(), "kamn-live-task-"));
	const agentAKey = resolve(root, "agent-a.key");
	const agentBKey = resolve(root, "agent-b.key");
	const agentCKey = resolve(root, "agent-c.key");
	const stopFile = resolve(root, "stop");
	await writeFile(agentAKey, "test-agent-a-key\n");
	await writeFile(agentBKey, "test-agent-b-key\n");
	await writeFile(agentCKey, "test-agent-c-key\n");
	await chmod(fixture, 0o755);
	return {
		root, stopFile,
		env: {
			KAMN_MVP_PI_RUN_ID: "test-run",
			KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS: "1000000",
			KAMN_MVP_LIVE_MCP_BINARY: fixture,
			KAMN_MVP_LIVE_MCP_ENDPOINT: "http://127.0.0.1:18278",
			KAMN_MVP_LIVE_MCP_AGENT_A_NAME: "agent-a",
			KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE: agentAKey,
			KAMN_MVP_LIVE_MCP_AGENT_B_NAME: "agent-b",
			KAMN_MVP_LIVE_MCP_AGENT_B_KEY_FILE: agentBKey,
			KAMN_MVP_LIVE_MCP_AGENT_C_NAME: "agent-c",
			KAMN_MVP_LIVE_MCP_AGENT_C_KEY_FILE: agentCKey,
			KAMN_MVP_FAKE_MCP_STOP_FILE: stopFile,
			...extra,
		},
	};
}

export function completionDigest(termsDigest: string): string {
	return createHash("sha256").update(`completed:${termsDigest}`).digest("hex");
}

export function projectedReceipt(receipt: Record<string, unknown>) {
	return {
		receipt_id: receipt.service_receipt_id, receipt_digest: receipt.service_receipt_digest,
		action: receipt.action, resource_id: receipt.resource_id, resulting_state: receipt.resulting_state,
	};
}
