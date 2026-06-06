import { spawnSync } from "node:child_process";
import { Buffer } from "node:buffer";

import { root } from "./config.mjs";
import { smokeState } from "./state.mjs";
import { expect } from "./assertions.mjs";

export function runTest(label, fn) {
  const startedCommands = smokeState.commandRecords.length;
  try {
    fn();
    smokeState.testResults.push({
      label,
      ok: true,
      commandCount: smokeState.commandRecords.length - startedCommands
    });
  } catch (error) {
    smokeState.testResults.push({
      label,
      ok: false,
      commandCount: smokeState.commandRecords.length - startedCommands,
      error
    });
    throw error;
  }
}

export function runCli(name, args, options = {}) {
  const result = spawnSync(smokeState.binaryPath, args, {
    cwd: root,
    input: options.stdin,
    encoding: "utf8",
    windowsHide: true,
    maxBuffer: 1024 * 1024 * 64
  });
  const record = {
    name,
    args,
    cwd: root,
    stdinSummary: options.stdinSummary ?? summarizeStdin(options.stdin),
    exitCode: result.status ?? 1,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
    assertions: []
  };
  smokeState.commandRecords.push(record);
  if (record.error) {
    expect(record, false, `process spawned successfully: ${record.error}`);
  }
  return record;
}

function summarizeStdin(stdin) {
  if (stdin === undefined || stdin === null) {
    return null;
  }
  const byteCount = Buffer.isBuffer(stdin) ? stdin.length : Buffer.byteLength(String(stdin), "utf8");
  const unit = byteCount === 1 ? "byte" : "bytes";
  return `${byteCount} ${unit} stdin`;
}
