import type { McpSession } from "./mcp-session.ts";
import { delay } from "./live-task-workflow-support.ts";

type JsonObject = Record<string, unknown>;
const MAX_CALL_ATTEMPTS = 4;

export async function callWithAntiSpamBackoff(
	session: McpSession,
	tool: string,
	fields: JsonObject,
	signal?: AbortSignal,
): Promise<JsonObject> {
	for (let attempt = 1; attempt <= MAX_CALL_ATTEMPTS; attempt += 1) {
		try { return await session.call(tool, fields, signal); } catch (error) {
			const waitMs = antiSpamWaitMs(error);
			if (waitMs === undefined || attempt === MAX_CALL_ATTEMPTS) throw error;
			await delay(waitMs, signal);
		}
	}
	throw new Error("Authenticated MCP call retries exhausted");
}

function antiSpamWaitMs(error: unknown): number | undefined {
	if (!(error instanceof Error)) return undefined;
	const suspension = error.message.match(/suspended by anti-spam policy until unix=(\d+)/);
	if (suspension) return Math.max(1000, Number(suspension[1]) * 1000 - Date.now() + 1000);
	const window = error.message.match(/anti-spam rate limit exceeded:.*window_seconds=(\d+)/);
	return window ? Number(window[1]) * 1000 + 1000 : undefined;
}
