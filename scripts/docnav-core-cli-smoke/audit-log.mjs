import fs from "node:fs";
import path from "node:path";

import { logDir, logPaths, root, tempRoot } from "./config.mjs";
import { smokeState } from "./state.mjs";

export function writeAuditLogs() {
  fs.mkdirSync(logDir, { recursive: true });
  const content = [
    "docnav core CLI smoke audit",
    `started: ${smokeState.startedAt.toISOString()}`,
    `completed: ${new Date().toISOString()}`,
    `cwd: ${root}`,
    `temp_root: ${tempRoot}`,
    `docnav_binary: ${smokeState.docnavBinaryPath ?? "(missing)"}`,
    `docnav_markdown_binary: ${smokeState.markdownBinaryPath ?? "(missing)"}`,
    "",
    "## Tests",
    ...smokeState.testResults.map((result) => {
      const status = result.ok ? "PASS" : "FAIL";
      const error = result.error ? `: ${result.error.message}` : "";
      return `${status} ${result.label} (${result.commandCount} command(s))${error}`;
    }),
    "",
    "## Commands",
    ...smokeState.commandRecords.flatMap(formatCommandRecord)
  ].join("\n");

  for (const logPath of logPaths) {
    fs.writeFileSync(logPath, `${content}\n`, "utf8");
  }
}

export function formatCommandRecord(record) {
  return [
    `### ${record.name}`,
    `$ ${quoteArg(smokeState.docnavBinaryPath ?? "docnav")} ${record.args.map(quoteArg).join(" ")}`.trimEnd(),
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

export function formatAssertions(assertions) {
  if (assertions.length === 0) {
    return ["  (none)"];
  }
  return assertions.map((assertion) => `  ${assertion.ok ? "PASS" : "FAIL"} ${assertion.summary}`);
}

export function printSuccessSummary() {
  console.log("");
  console.log("Docnav Core CLI Smoke");
  console.log("Status: passed");
  console.log(`Commands: ${smokeState.commandRecords.length}`);
  console.log("");
  console.log("Log:");
  console.log(`  - ${relativeLogPath(logPaths[0])}`);
  console.log("");
}

export function printFailureSummary(error) {
  console.error("");
  console.error("Docnav Core CLI Smoke");
  console.error("Status: failed");
  console.error(`Failure: ${error.message}`);
  console.error("");
  console.error("Log:");
  console.error(`  - ${relativeLogPath(logPaths[0])}`);
  console.error("");
}

function quoteArg(value) {
  if (/^[A-Za-z0-9_./:=@+\-\\]+$/.test(value)) {
    return value;
  }
  return JSON.stringify(value);
}

function relativeLogPath(logPath) {
  return path.relative(root, logPath).replaceAll(path.sep, "/");
}

