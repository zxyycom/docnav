import assert from "node:assert/strict";
import test from "node:test";
import { formatCommandOutput, formatError } from "./approval-session-output.mjs";

test("formats a running session without internal process details", () => {
  const output = formatCommandOutput("start", {
    sessionId: "00000000-0000-4000-8000-000000000001",
    processId: 42,
    processRunning: true,
    state: {
      status: "running",
      claudeVersion: "2.1.173 (Claude Code)",
    },
    pendingRequests: [],
    result: null,
    stderrTail: [],
  });

  assert.equal(
    output,
    [
      "status: running",
      "session: 00000000-0000-4000-8000-000000000001",
      "",
    ].join("\n"),
  );
  assert.doesNotMatch(output, /sessionDirectory|processId|processRunning|stderrTail/u);
});

test("formats pending approval details", () => {
  const output = formatCommandOutput("status", {
    sessionId: "00000000-0000-4000-8000-000000000001",
    state: { status: "awaiting_approval" },
    pendingRequests: [
      {
        requestId: "request-id",
        toolName: "Bash",
        input: { command: "rtk --version" },
        decisionReason: "This command requires approval",
      },
    ],
  });

  assert.match(output, /^status: awaiting_approval$/mu);
  assert.match(output, /^session: 00000000-0000-4000-8000-000000000001$/mu);
  assert.match(output, /^request: request-id$/mu);
  assert.match(output, /^tool: Bash$/mu);
  assert.match(output, /^command:$/mu);
  assert.match(output, /^  rtk --version$/mu);
  assert.match(output, /^reason: This command requires approval$/mu);
});

test("keeps non-Bash approval inputs exact", () => {
  const output = formatCommandOutput("status", {
    state: { status: "awaiting_approval" },
    pendingRequests: [
      {
        requestId: "request-id",
        toolName: "Read",
        input: { file_path: "fixtures\\file.txt" },
      },
    ],
  });

  assert.match(output, /^input:$/mu);
  assert.match(output, /"file_path": "fixtures\\\\file.txt"/u);
});

test("formats final results and decisions", () => {
  assert.equal(
    formatCommandOutput("status", {
      sessionId: "00000000-0000-4000-8000-000000000001",
      state: { status: "completed" },
      pendingRequests: [],
      result: { text: "rtk 0.42.3" },
    }),
    "status: completed\nsession: 00000000-0000-4000-8000-000000000001\nresult:\n  rtk 0.42.3\n",
  );
  assert.equal(
    formatCommandOutput("approve", {
      behavior: "allow",
      requestId: "request-id",
    }),
    "approved: request-id\n",
  );
  assert.equal(formatError(new Error("broken")), "error: broken\n");
});
