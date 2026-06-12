import { randomUUID } from "node:crypto";
import {
  appendFile,
  mkdir,
  readFile,
  readdir,
  rename,
} from "node:fs/promises";
import path from "node:path";
import { query } from "@anthropic-ai/claude-agent-sdk";
import {
  appendJsonLine,
  delay,
  readJson,
  sessionPaths,
  timestamp,
  writeJsonAtomic,
} from "./approval-session-store.mjs";

const sessionDirectory = requireEnvironment("CLAUDE_APPROVAL_SESSION_DIR");
const workingDirectory = requireEnvironment(
  "CLAUDE_APPROVAL_WORKING_DIRECTORY",
);
const promptFile = requireEnvironment("CLAUDE_APPROVAL_PROMPT_FILE");
const claudeExecutable = requireEnvironment(
  "CLAUDE_APPROVAL_CLAUDE_EXECUTABLE",
);
const claudeVersion = requireEnvironment("CLAUDE_APPROVAL_CLAUDE_VERSION");
const permissionMode =
  process.env.CLAUDE_APPROVAL_PERMISSION_MODE || "auto";

const files = sessionPaths(sessionDirectory);
const abortController = new AbortController();
const stateContext = {
  claudeExecutable,
  claudeVersion,
  permissionMode,
  workingDirectory,
};
let stopReason = null;
let sdkSessionId = null;

function requireEnvironment(name) {
  const value = process.env[name];
  if (!value) {
    throw new Error(`Missing required environment variable: ${name}`);
  }
  return value;
}

async function writeState(status, details = {}) {
  await writeJsonAtomic(files.state, {
    status,
    updatedAt: timestamp(),
    ...stateContext,
    ...details,
  });
}

async function appendEvent(type, details = {}) {
  await appendJsonLine(files.events, {
    type,
    timestamp: timestamp(),
    ...details,
  });
}

async function watchForStop() {
  while (!abortController.signal.aborted) {
    const request = await readJson(files.stop);
    if (request) {
      stopReason = request.reason || "Codex stopped this Claude session.";
      abortController.abort(new Error(stopReason));
      return;
    }
    await delay(200, abortController.signal).catch(() => {});
  }
}

async function archiveApprovalFiles(requestId, requestFile, decisionFile) {
  await rename(
    requestFile,
    path.join(files.resolved, `${requestId}.request.json`),
  );
  await rename(
    decisionFile,
    path.join(files.resolved, `${requestId}.decision.json`),
  );
}

async function archiveStoppedRequests() {
  for (const entry of await readdir(files.requests, {
    withFileTypes: true,
  })) {
    if (!entry.isFile() || !entry.name.endsWith(".json")) {
      continue;
    }
    const requestId = entry.name.slice(0, -".json".length);
    await rename(
      path.join(files.requests, entry.name),
      path.join(files.resolved, `${requestId}.stopped.request.json`),
    );
  }
}

async function canUseTool(toolName, input, context) {
  const requestId = randomUUID();
  const requestFile = path.join(files.requests, `${requestId}.json`);
  const decisionFile = path.join(files.decisions, `${requestId}.json`);
  const request = {
    requestId,
    toolUseId: context.toolUseID,
    toolName,
    input,
    requestedAt: timestamp(),
    title: context.title,
    displayName: context.displayName,
    description: context.description,
    decisionReason: context.decisionReason,
    blockedPath: context.blockedPath,
    agentId: context.agentID,
    suggestions: context.suggestions,
  };

  await writeJsonAtomic(requestFile, request);
  await appendEvent("approval_requested", request);
  await writeState("awaiting_approval", {
    requestId,
    toolName,
    sessionId: sdkSessionId,
  });

  while (true) {
    const decision = await readJson(decisionFile);
    if (!decision) {
      await delay(200, context.signal, "Approval request aborted");
      continue;
    }

    if (decision.requestId !== requestId) {
      throw new Error(
        `Approval decision requestId mismatch: expected ${requestId}`,
      );
    }

    if (decision.behavior !== "allow" && decision.behavior !== "deny") {
      throw new Error(
        `Unsupported approval behavior for ${requestId}: ${decision.behavior}`,
      );
    }

    await archiveApprovalFiles(requestId, requestFile, decisionFile);
    await appendEvent("approval_resolved", {
      requestId,
      toolName,
      behavior: decision.behavior,
      reason: decision.reason,
    });
    await writeState("running", {
      lastResolvedRequestId: requestId,
      lastDecision: decision.behavior,
      sessionId: sdkSessionId,
    });

    if (decision.behavior === "allow") {
      return {
        behavior: "allow",
        updatedInput: decision.updatedInput ?? input,
      };
    }

    return {
      behavior: "deny",
      message:
        decision.message ||
        decision.reason ||
        "Codex denied this tool request.",
    };
  }
}

async function main() {
  await mkdir(files.requests, { recursive: true });
  await mkdir(files.decisions, { recursive: true });
  await mkdir(files.resolved, { recursive: true });

  const prompt = await readFile(promptFile, "utf8");
  await writeState("running");
  await appendEvent("session_started", stateContext);

  const stopWatcher = watchForStop();
  let finalResult = null;
  try {
    for await (const message of query({
      prompt,
      options: {
        abortController,
        cwd: workingDirectory,
        pathToClaudeCodeExecutable: claudeExecutable,
        permissionMode,
        settingSources: ["user", "project", "local"],
        systemPrompt: { type: "preset", preset: "claude_code" },
        tools: { type: "preset", preset: "claude_code" },
        canUseTool,
        stderr: (data) => {
          void appendFile(files.stderr, data, "utf8");
        },
      },
    })) {
      await appendJsonLine(files.messages, message);
      if (
        message.type === "system" &&
        message.subtype === "init" &&
        message.session_id
      ) {
        sdkSessionId = message.session_id;
        await writeState("running", { sessionId: sdkSessionId });
      }
      if (message.type === "result") {
        finalResult = message;
      }
    }
  } finally {
    abortController.abort();
    await stopWatcher.catch(() => {});
  }

  if (!finalResult) {
    throw new Error("Claude Agent SDK ended without a result message.");
  }

  await writeJsonAtomic(files.result, finalResult);
  await appendEvent("session_completed", {
    subtype: finalResult.subtype,
    sessionId: finalResult.session_id,
  });
  await writeState(
    finalResult.subtype === "success" ? "completed" : "failed",
    {
      resultSubtype: finalResult.subtype,
      sessionId: finalResult.session_id,
    },
  );
}

main().catch(async (error) => {
  if (stopReason) {
    await archiveStoppedRequests().catch(() => {});
    await appendEvent("session_stopped", { reason: stopReason }).catch(() => {});
    await writeJsonAtomic(files.result, {
      type: "bridge_stopped",
      message: stopReason,
    }).catch(() => {});
    await writeState("stopped", { reason: stopReason }).catch(() => {});
    return;
  }

  const failure = {
    name: error?.name || "Error",
    message: error?.message || String(error),
    stack: error?.stack,
  };
  await appendEvent("session_failed", failure).catch(() => {});
  await writeJsonAtomic(files.result, {
    type: "bridge_error",
    ...failure,
  }).catch(() => {});
  await writeState("failed", failure).catch(() => {});
  process.exitCode = 1;
});
