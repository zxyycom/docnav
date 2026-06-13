import { randomUUID } from "node:crypto";
import {
  appendFile,
  readFile,
  readdir,
  rename,
  rm,
  writeFile,
} from "node:fs/promises";
import os from "node:os";
import path from "node:path";

export const SESSION_ROOT = path.join(os.tmpdir(), "codex-claude-approval");
export const FINAL_STATUSES = new Set(["completed", "failed", "stopped"]);

export function sessionPaths(sessionDirectory) {
  return {
    requests: path.join(sessionDirectory, "requests"),
    decisions: path.join(sessionDirectory, "decisions"),
    resolved: path.join(sessionDirectory, "resolved"),
    state: path.join(sessionDirectory, "state.json"),
    result: path.join(sessionDirectory, "result.json"),
    events: path.join(sessionDirectory, "events.ndjson"),
    messages: path.join(sessionDirectory, "messages.ndjson"),
    stderr: path.join(sessionDirectory, "stderr.log"),
    stop: path.join(sessionDirectory, "stop.json"),
    pid: path.join(sessionDirectory, "pid"),
  };
}

export function timestamp() {
  return new Date().toISOString();
}

export async function writeJsonAtomic(filePath, value) {
  const temporaryPath = `${filePath}.${randomUUID()}.tmp`;
  await writeFile(temporaryPath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
  try {
    await rename(temporaryPath, filePath);
  } catch (error) {
    if (error?.code !== "EEXIST" && error?.code !== "EPERM") {
      throw error;
    }
    await rm(filePath, { force: true });
    await rename(temporaryPath, filePath);
  }
}

export async function readJson(filePath) {
  try {
    return JSON.parse(await readFile(filePath, "utf8"));
  } catch (error) {
    if (error?.code === "ENOENT" || error instanceof SyntaxError) {
      return null;
    }
    throw error;
  }
}

export async function appendJsonLine(filePath, value) {
  await appendFile(filePath, `${JSON.stringify(value)}\n`, "utf8");
}

export async function listJsonFiles(directory) {
  return (await readdir(directory, { withFileTypes: true }))
    .filter((entry) => entry.isFile() && entry.name.endsWith(".json"))
    .map((entry) => entry.name);
}

export function delay(milliseconds, signal, abortMessage = "Operation aborted") {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) {
      reject(signal.reason || new Error(abortMessage));
      return;
    }

    const timer = setTimeout(resolve, milliseconds);
    signal?.addEventListener(
      "abort",
      () => {
        clearTimeout(timer);
        reject(signal.reason || new Error(abortMessage));
      },
      { once: true },
    );
  });
}
