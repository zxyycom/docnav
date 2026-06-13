function appendBlock(lines, label, value) {
  if (value === undefined || value === null || value === "") return;
  const text =
    typeof value === "string" ? value : JSON.stringify(value, null, 2);
  lines.push(`${label}:`);
  lines.push(...text.split(/\r?\n/u).map((line) => `  ${line}`));
}

function appendBashInput(lines, input) {
  appendBlock(lines, "command", input.command);
  if (input.description) {
    lines.push(`description: ${input.description}`);
  }
  if (input.timeout !== undefined) {
    lines.push(`timeout: ${input.timeout}`);
  }
}

function appendToolInput(lines, request) {
  if (request.toolName === "Bash" && request.input?.command) {
    appendBashInput(lines, request.input);
    return;
  }
  appendBlock(lines, "input", request.input);
}

function formatSnapshot(snapshot) {
  const status = snapshot.state?.status || "unknown";
  const lines = [`status: ${status}`];
  if (snapshot.sessionId) {
    lines.push(`session: ${snapshot.sessionId}`);
  }

  for (const request of snapshot.pendingRequests || []) {
    lines.push("");
    lines.push(`request: ${request.requestId}`);
    lines.push(`tool: ${request.toolName}`);
    appendToolInput(lines, request);
    if (request.decisionReason) {
      lines.push(`reason: ${request.decisionReason}`);
    }
  }

  const result = snapshot.result;
  appendBlock(lines, "result", result?.text ?? result?.message);
  appendBlock(lines, "errors", result?.errors);

  if (!result?.message && snapshot.state?.reason) {
    lines.push(`reason: ${snapshot.state.reason}`);
  }
  if (status === "failed") {
    appendBlock(lines, "stderr", snapshot.stderrTail?.join("\n"));
  }

  return `${lines.join("\n")}\n`;
}

function formatDecision(decision) {
  const action = decision.behavior === "allow" ? "approved" : "denied";
  return `${action}: ${decision.requestId}\n`;
}

export function formatCommandOutput(command, value) {
  if (command === "approve" || command === "deny") {
    return formatDecision(value);
  }
  return formatSnapshot(value);
}

export function formatError(error) {
  return `error: ${error?.message || String(error)}\n`;
}
