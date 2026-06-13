import { execFile, spawn } from "node:child_process";
import { randomUUID } from "node:crypto";
import { mkdir, readFile, readdir, realpath, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import {
  delay,
  FINAL_STATUSES,
  listJsonFiles,
  readJson,
  SESSION_ROOT,
  sessionPaths,
  timestamp,
  writeJsonAtomic,
} from "./approval-session-store.mjs";

const runtimeDirectory = path.dirname(fileURLToPath(import.meta.url));
const bridgePath = path.join(runtimeDirectory, "claude-approval-bridge.mjs");
const permissionModes = new Set(["acceptEdits", "default", "plan", "auto"]);
const execFileAsync = promisify(execFile);
const UUID_PATTERN =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/iu;

async function pathExists(filePath) {
  try {
    await realpath(filePath);
    return true;
  } catch (error) {
    if (error?.code === "ENOENT") return false;
    throw error;
  }
}

async function findExecutable(command) {
  const locator = process.platform === "win32" ? "where.exe" : "which";
  try {
    const { stdout } = await execFileAsync(locator, [command], {
      encoding: "utf8",
      windowsHide: true,
    });
    return stdout
      .split(/\r?\n/u)
      .map((line) => line.trim())
      .find(Boolean);
  } catch (error) {
    if (error?.code === "ENOENT" || error?.code === 1) return null;
    throw error;
  }
}

async function resolveClaudeExecutable(explicitExecutable) {
  const requested = explicitExecutable || "claude";
  const located = path.isAbsolute(requested)
    ? requested
    : await findExecutable(requested);
  if (!located) {
    throw new Error(
      `System Claude Code executable was not found: ${requested}. Install it or pass --claude-executable <path>.`,
    );
  }
  try {
    return await realpath(located);
  } catch (error) {
    if (error?.code === "ENOENT") {
      throw new Error(`Claude Code executable does not exist: ${located}`);
    }
    throw error;
  }
}

async function readClaudeVersion(claudeExecutable) {
  const { stdout } = await execFileAsync(claudeExecutable, ["--version"], {
    encoding: "utf8",
    timeout: 10_000,
    windowsHide: true,
  });
  return stdout.trim();
}

function requireSessionId(value) {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error("Provide --session-id.");
  }
  if (!UUID_PATTERN.test(value)) {
    throw new Error("--session-id must be a UUID.");
  }
  return value;
}

async function readLatestSessionId() {
  let latest;
  try {
    latest = (await readFile(path.join(SESSION_ROOT, "latest.txt"), "utf8")).trim();
  } catch (error) {
    if (error?.code === "ENOENT") {
      throw new Error("No Claude approval session has been started.");
    }
    throw error;
  }
  return requireSessionId(path.basename(latest));
}

async function resolveSessionDirectory(sessionId) {
  const resolvedSessionId = sessionId
    ? requireSessionId(sessionId)
    : await readLatestSessionId();
  const sessionDirectory = path.join(SESSION_ROOT, resolvedSessionId);
  const resolvedRoot = path.resolve(SESSION_ROOT);
  if (!(await pathExists(sessionDirectory))) {
    throw new Error(`Session does not exist: ${resolvedSessionId}`);
  }
  const resolvedDirectory = await realpath(sessionDirectory);
  const relative = path.relative(resolvedRoot, resolvedDirectory);
  if (
    relative === "" ||
    relative.startsWith("..") ||
    path.isAbsolute(relative)
  ) {
    throw new Error(`Session directory must be a child of: ${resolvedRoot}`);
  }
  return resolvedDirectory;
}

async function listSessionDirectories() {
  let entries;
  try {
    entries = await readdir(SESSION_ROOT, { withFileTypes: true });
  } catch (error) {
    if (error?.code === "ENOENT") return [];
    throw error;
  }
  return entries
    .filter((entry) => entry.isDirectory() && UUID_PATTERN.test(entry.name))
    .map((entry) => path.join(SESSION_ROOT, entry.name));
}

async function resolveRequestSessionDirectory(requestId) {
  if (typeof requestId !== "string" || !UUID_PATTERN.test(requestId)) {
    throw new Error("--request-id must be a UUID.");
  }
  const matches = [];
  await Promise.all(
    (await listSessionDirectories()).map(async (sessionDirectory) => {
      const files = sessionPaths(sessionDirectory);
      const requestFile = path.join(files.requests, `${requestId}.json`);
      const decisionFile = path.join(files.decisions, `${requestId}.json`);
      if (
        (await pathExists(requestFile)) &&
        !(await pathExists(decisionFile))
      ) {
        matches.push(sessionDirectory);
      }
    }),
  );
  if (matches.length === 0) {
    throw new Error(`Pending request does not exist: ${requestId}`);
  }
  if (matches.length > 1) {
    throw new Error(`Pending request is ambiguous: ${requestId}`);
  }
  return matches[0];
}

function isProcessRunning(processId) {
  if (!Number.isInteger(processId) || processId <= 0) return false;
  try {
    process.kill(processId, 0);
    return true;
  } catch (error) {
    if (error?.code === "EPERM") return true;
    if (error?.code === "ESRCH") return false;
    throw error;
  }
}

function summarizeResult(result) {
  if (!result) return null;
  return {
    type: result.type,
    subtype: result.subtype,
    isError: result.is_error,
    text: result.result,
    message: result.message,
    errors: result.errors,
    sessionId: result.session_id,
    permissionDenials: result.permission_denials,
    durationMs: result.duration_ms,
    totalCostUsd: result.total_cost_usd,
  };
}

async function readTail(filePath, lineCount) {
  try {
    const lines = (await readFile(filePath, "utf8")).split(/\r?\n/u);
    if (lines.at(-1) === "") lines.pop();
    return lines.slice(-lineCount);
  } catch (error) {
    if (error?.code === "ENOENT") return [];
    throw error;
  }
}

async function getSessionSnapshot(sessionDirectory) {
  const files = sessionPaths(sessionDirectory);
  const [state, result, requestNames, decisionNames, processIdText, stderrTail] =
    await Promise.all([
      readJson(files.state),
      readJson(files.result),
      listJsonFiles(files.requests),
      listJsonFiles(files.decisions),
      readFile(files.pid, "utf8").catch((error) => {
        if (error?.code === "ENOENT") return null;
        throw error;
      }),
      readTail(files.stderr, 20),
    ]);

  const decided = new Set(decisionNames);
  const pendingRequests = (
    await Promise.all(
      requestNames
        .filter((name) => !decided.has(name))
        .map((name) => readJson(path.join(files.requests, name))),
    )
  ).filter(Boolean);
  pendingRequests.sort((left, right) =>
    String(left.requestedAt).localeCompare(String(right.requestedAt)),
  );

  const processId = processIdText === null ? null : Number(processIdText.trim());
  return {
    sessionId: path.basename(sessionDirectory),
    processId,
    processRunning: FINAL_STATUSES.has(state?.status)
      ? false
      : isProcessRunning(processId),
    state,
    pendingRequests,
    result: summarizeResult(result),
    stderrTail,
  };
}

async function waitForSnapshot(sessionDirectory, waitSeconds, isReady) {
  const deadline = Date.now() + waitSeconds * 1000;
  while (true) {
    const snapshot = await getSessionSnapshot(sessionDirectory);
    if (isReady(snapshot) || Date.now() >= deadline) return snapshot;
    await delay(250);
  }
}

function statusReady(snapshot) {
  return (
    snapshot.pendingRequests.length > 0 ||
    FINAL_STATUSES.has(snapshot.state?.status)
  );
}

export async function startSession({
  workingDirectory,
  prompt,
  promptFile,
  permissionMode = "auto",
  claudeExecutable,
}) {
  const resolvedWorkingDirectory = await realpath(workingDirectory);
  const resolvedClaudeExecutable = await resolveClaudeExecutable(
    claudeExecutable,
  );
  const claudeVersion = await readClaudeVersion(resolvedClaudeExecutable);
  if (!permissionModes.has(permissionMode)) {
    throw new Error(
      "--permission-mode must be one of: auto, acceptEdits, default, plan.",
    );
  }

  const taskPrompt = promptFile ? await readFile(promptFile, "utf8") : prompt;
  if (typeof taskPrompt !== "string" || taskPrompt.length === 0) {
    throw new Error("Provide --prompt or --prompt-file.");
  }
  if (!(await pathExists(bridgePath))) {
    throw new Error(`Claude approval bridge is missing: ${bridgePath}`);
  }
  if (!(await pathExists(path.join(runtimeDirectory, "node_modules")))) {
    throw new Error(
      `Install the bridge runtime first: cd "${runtimeDirectory}" && pnpm install --frozen-lockfile`,
    );
  }

  const sessionId = randomUUID();
  const sessionDirectory = path.join(SESSION_ROOT, sessionId);
  const files = sessionPaths(sessionDirectory);
  const storedPromptFile = path.join(sessionDirectory, "prompt.txt");
  await Promise.all([
    mkdir(files.requests, { recursive: true }),
    mkdir(files.decisions, { recursive: true }),
    mkdir(files.resolved, { recursive: true }),
  ]);
  await writeFile(storedPromptFile, taskPrompt, "utf8");
  await writeJsonAtomic(path.join(sessionDirectory, "metadata.json"), {
    sessionId,
    workingDirectory: resolvedWorkingDirectory,
    permissionMode,
    claudeExecutable: resolvedClaudeExecutable,
    claudeVersion,
    createdAt: timestamp(),
  });

  const child = spawn(process.execPath, [bridgePath], {
    cwd: runtimeDirectory,
    detached: true,
    stdio: "ignore",
    windowsHide: true,
    env: {
      ...process.env,
      CLAUDE_APPROVAL_SESSION_DIR: sessionDirectory,
      CLAUDE_APPROVAL_WORKING_DIRECTORY: resolvedWorkingDirectory,
      CLAUDE_APPROVAL_PROMPT_FILE: storedPromptFile,
      CLAUDE_APPROVAL_PERMISSION_MODE: permissionMode,
      CLAUDE_APPROVAL_CLAUDE_EXECUTABLE: resolvedClaudeExecutable,
      CLAUDE_APPROVAL_CLAUDE_VERSION: claudeVersion,
    },
  });
  child.unref();

  await writeFile(files.pid, String(child.pid), "utf8");
  await mkdir(SESSION_ROOT, { recursive: true });
  await writeFile(path.join(SESSION_ROOT, "latest.txt"), sessionId, "utf8");
  return waitForSnapshot(sessionDirectory, 2, statusReady);
}

export async function getStatus({ sessionId, waitSeconds = 0 }) {
  return waitForSnapshot(
    await resolveSessionDirectory(sessionId),
    waitSeconds,
    statusReady,
  );
}

export async function decideRequest({
  requestId,
  behavior,
  reason,
  message,
  updatedInput,
}) {
  const resolvedDirectory = await resolveRequestSessionDirectory(requestId);
  const files = sessionPaths(resolvedDirectory);
  const requestFile = path.join(files.requests, `${requestId}.json`);
  if (!(await pathExists(requestFile))) {
    throw new Error(`Pending request does not exist: ${requestId}`);
  }

  const decision = {
    requestId,
    behavior,
    reason,
    decidedAt: timestamp(),
  };
  if (updatedInput !== undefined) decision.updatedInput = updatedInput;
  if (behavior === "deny") {
    decision.message = message || "Codex denied this tool request.";
  }

  await writeJsonAtomic(
    path.join(files.decisions, `${requestId}.json`),
    decision,
  );
  return { ...decision, sessionId: path.basename(resolvedDirectory) };
}

export async function stopSession({ sessionId, reason }) {
  const resolvedDirectory = await resolveSessionDirectory(sessionId);
  const initialSnapshot = await getSessionSnapshot(resolvedDirectory);
  if (
    !initialSnapshot.processRunning ||
    FINAL_STATUSES.has(initialSnapshot.state?.status)
  ) {
    return initialSnapshot;
  }

  await writeJsonAtomic(sessionPaths(resolvedDirectory).stop, {
    requestedAt: timestamp(),
    reason: reason || "Codex stopped this Claude session.",
  });

  const snapshot = await waitForSnapshot(
    resolvedDirectory,
    15,
    (current) =>
      !current.processRunning || current.state?.status === "stopped",
  );
  if (!snapshot.processRunning || snapshot.state?.status === "stopped") {
    return snapshot;
  }
  throw new Error(
    `Claude bridge did not stop within 15 seconds: ${resolvedDirectory}`,
  );
}
