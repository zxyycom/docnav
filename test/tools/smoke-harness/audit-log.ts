import fs from "node:fs";

import type { AssertionRecord, CommandRecord, SmokeState } from "../smoke-harness.ts";
import { formatTestResult } from "./reports.ts";

interface SmokeAuditLogOptions {
  auditMetadata: () => string[];
  auditTitle: string;
  binaryFallback: string;
  binaryPath: () => string | null;
  logDir: string;
  logPaths: string[];
  root: string;
  safeArgPattern: RegExp;
  state: SmokeState;
}

export function createSmokeAuditLog(options: SmokeAuditLogOptions) {
  const {
    auditMetadata,
    auditTitle,
    binaryFallback,
    binaryPath,
    logDir,
    logPaths,
    root,
    safeArgPattern,
    state
  } = options;

  function formatCommandRecord(record: CommandRecord) {
    return [
      `### ${record.name}`,
      `$ ${quoteArg(binaryPath() ?? binaryFallback)} ${record.args.map(quoteArg).join(" ")}`.trimEnd(),
      `cwd: ${record.cwd}`,
      `stdin: ${record.stdinSummary ?? "(none)"}`,
      `exit: ${record.exitCode}`,
      record.signal ? `signal: ${record.signal}` : null,
      record.error ? `spawn_error: ${record.error}` : null,
      "stdout:",
      record.stdout.length > 0 ? record.stdout : "(empty)",
      "stderr:",
      record.stderr.length > 0 ? record.stderr : "(empty)",
      "assertions:",
      ...formatAssertions(record.assertions),
      ""
    ].filter((line) => line !== null);
  }

  function writeAuditLogs() {
    fs.mkdirSync(logDir, { recursive: true });
    const content = [
      auditTitle,
      `started: ${state.startedAt.toISOString()}`,
      `completed: ${new Date().toISOString()}`,
      `cwd: ${root}`,
      ...auditMetadata(),
      "",
      "## Tests",
      ...state.testResults.map(formatTestResult),
      "",
      "## Commands",
      ...state.commandRecords.flatMap(formatCommandRecord)
    ].join("\n");

    for (const logPath of logPaths) {
      fs.writeFileSync(logPath, `${content}\n`, "utf8");
    }
  }

  function quoteArg(value: string) {
    return safeArgPattern.test(value) ? value : JSON.stringify(value);
  }

  return {
    formatCommandRecord,
    writeAuditLogs
  };
}

export function formatAssertions(assertions: readonly AssertionRecord[]): string[] {
  if (assertions.length === 0) {
    return ["  (none)"];
  }
  return assertions.map((assertion) => `  ${assertion.ok ? "PASS" : "FAIL"} ${assertion.summary}`);
}
