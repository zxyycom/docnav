import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { booleanOption, parsePositiveInteger, parseScriptArgs, stringOption } from "../tools/args.ts";
import { runParallelTasks } from "../tools/parallel-task-runner/index.ts";
import { processFailure, processFailureFromResult, runProcess } from "../tools/process.ts";
import type { ProcessFailure } from "../tools/process.ts";
import { readJsonFile } from "../tools/fs.ts";
import { toSlashPath } from "../tools/path.ts";
import { errorMessage } from "../tools/errors.ts";
import { isRecord } from "../tools/type-guards.ts";
import {
  PROFILE_FULL,
  PROFILE_REQUIRED,
  asCheckTask,
  checks,
  checksForProfile,
  parseProfile,
  profiles,
  reportCountForChecks,
  visibleOutputLines,
  isIgnoredOutput
} from "./checks/index.ts";
import type { CheckTask, Profile } from "./checks/index.ts";
import {
  createReportCompletionTracker,
  formatCompletionLine,
  formatDurationMs
} from "./results.ts";
import type { CheckResult, CompletionResult } from "./results.ts";

export {
  PROFILE_FULL,
  PROFILE_REQUIRED,
  checks,
  checksForProfile,
  formatCompletionLine,
  formatDurationMs,
  profiles,
  reportCountForChecks,
  visibleOutputLines,
  isIgnoredOutput
};

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const logDir = path.join(root, ".log", "verify-docnav-workspace");
const MAX_BUFFER = 1024 * 1024 * 64;

type StringMap = Record<string, string>;

interface VerificationOptions {
  help: boolean;
  profile: Profile;
  concurrency: number | undefined;
}

interface CheckExecutionData {
  endedAtMs: number;
  error?: unknown;
  exitCode: number;
  ok: boolean;
  startedAtMs: number;
  stderr: string;
  stdout: string;
}

interface SummaryInput {
  profile: Profile;
  totalChecks: number;
  completedResults: readonly CompletionResult[];
  totalDurationMs: number;
  logPaths: readonly string[];
}

if (isMainModule()) {
  void main();
}

export function parseArgs(argv: string[]): VerificationOptions {
  const parsed = parseScriptArgs({
    args: argv,
    options: {
      concurrency: { type: "string" },
      help: { type: "boolean", short: "h" },
      profile: { type: "string" }
    }
  });

  return {
    help: booleanOption(parsed.values, "help"),
    profile: parseProfile(stringOption(parsed.values, "profile") ?? PROFILE_FULL),
    concurrency: resolveVerificationConcurrency(stringOption(parsed.values, "concurrency"))
  };
}

export function resolveVerificationConcurrency(value = process.env.DOCNAV_VERIFY_CONCURRENCY) {
  if (value === undefined || value === null || value === "") {
    return undefined;
  }
  return parsePositiveInteger(value, "verification concurrency");
}

async function main() {
  try {
    const options = parseArgs(process.argv.slice(2));
    if (options.help) {
      printUsage(console.log);
      process.exitCode = 0;
      return;
    }
    process.exitCode = await runVerification(options);
  } catch (error: unknown) {
    console.error(errorMessage(error));
    printUsage(console.error);
    process.exitCode = 2;
  }
}

async function runVerification({ profile, concurrency }: VerificationOptions): Promise<number> {
  const selectedChecks = checksForProfile(profile);
  const totalReports = reportCountForChecks(selectedChecks);
  const completeReport = createReportCompletionTracker(selectedChecks);
  const logPaths = createLogPaths();
  initializeLogs(logPaths, profile, totalReports, selectedChecks.length);

  const startedAtMs = Date.now();
  const completedReports: CompletionResult[] = [];

  printHeader(profile, totalReports);
  await runParallelTasks<CheckResult>(selectedChecks, {
    concurrency,
    execute: (task) => executeCheck(asCheckTask(task)),
    onComplete: (result) => {
      appendLog(logPaths, result);
      const report = completeReport(result);
      if (report) {
        completedReports.push(report);
        console.log(formatCompletionLine(report));
      }
    }
  });

  const failures = completedReports.filter((result) => !result.ok);
  const totalDurationMs = Date.now() - startedAtMs;
  finalizeLogs(logPaths, totalDurationMs);

  printSummary({
    profile,
    totalChecks: totalReports,
    completedResults: completedReports,
    totalDurationMs,
    logPaths
  });

  return failures.length > 0 ? 1 : 0;
}

async function executeCheck(check: CheckTask): Promise<CheckResult> {
  const startedAtMs = Date.now();
  const result = await runProcess({
    args: check.args,
    command: check.command,
    cwd: root,
    env: environmentForCheck(check),
    maxBuffer: MAX_BUFFER
  });
  const failure = processFailureFromResult(result);
  return buildCheckResult(check, {
    ok: failure === null,
    exitCode: failure === null ? 0 : normalizeExitCode(failure),
    stdout: result.stdout,
    stderr: result.stderr,
    error: failure ?? undefined,
    startedAtMs,
    endedAtMs: Date.now()
  });
}

function buildCheckResult(check: CheckTask, data: CheckExecutionData): CheckResult {
  return {
    check,
    ok: data.ok,
    exitCode: data.exitCode,
    error: data.error === undefined ? null : processFailure(data.error),
    stdout: data.stdout,
    stderr: data.stderr,
    combinedOutput: combinedProcessOutput(data),
    durationMs: data.endedAtMs - data.startedAtMs,
    startedAtMs: data.startedAtMs,
    endedAtMs: data.endedAtMs
  };
}

function combinedProcessOutput({ stdout, stderr }: Pick<CheckExecutionData, "stderr" | "stdout">): string {
  return [stdout, stderr].filter(Boolean).join("\n");
}

function normalizeExitCode(error: ProcessFailure): number {
  return typeof error?.code === "number" ? error.code : 1;
}

function environmentForCheck(check: CheckTask): NodeJS.ProcessEnv {
  return {
    ...process.env,
    ...readEnvFile(check.envFile),
    ...(check.env ?? {})
  };
}

function readEnvFile(envFile: string | undefined): StringMap {
  if (!envFile) {
    return {};
  }
  const envPath = path.resolve(root, envFile);
  const parsed = readJsonFile(envPath);
  if (!isRecord(parsed)) {
    throw new Error(`env file must contain an object: ${envFile}`);
  }
  return Object.fromEntries(
    Object.entries(parsed).map(([key, value]) => [key, String(value)])
  );
}

function printHeader(profile: Profile, totalChecks: number): void {
  console.log("");
  console.log("Docnav Workspace Verification");
  console.log(`Profile: ${profile}`);
  console.log(`Total checks: ${totalChecks}`);
  console.log("");
  console.log("Checks:");
}

function printSummary({
  profile,
  totalChecks,
  completedResults,
  totalDurationMs,
  logPaths
}: SummaryInput): void {
  console.log("");
  console.log("Summary:");
  console.log(`  status: ${completedResults.some((result) => !result.ok) ? "failed" : "passed"}`);
  console.log(`  profile: ${profile}`);
  console.log(`  total checks: ${totalChecks}`);
  console.log(`  passed: ${completedResults.filter((result) => result.ok).length}`);
  console.log(`  failed: ${completedResults.filter((result) => !result.ok).length}`);
  console.log(`  duration: ${formatDurationMs(totalDurationMs)}`);
  console.log(`  log: ${relativeLogPath(logPaths[0])}`);
  console.log("");
}

function appendLog(logPaths: readonly string[], result: CheckResult): void {
  const section = [
    `## ${result.check.label}`,
    `$ ${formatCommandLine(result.check.command, result.check.args)}`,
    `exit: ${result.exitCode}`,
    `duration: ${formatDurationMs(result.durationMs)}`,
    result.error ? `process_error: ${result.error.message}` : null,
    "",
    result.combinedOutput || "(no output)",
    "",
    ""
  ]
    .filter((line) => line !== null)
    .join("\n");

  for (const logPath of logPaths) {
    fs.appendFileSync(logPath, section, "utf8");
  }
}

function formatCommandLine(command: string, args: readonly string[] = []): string {
  return [command, ...args].map(quoteCommandArg).join(" ");
}

function quoteCommandArg(value: string): string {
  const text = String(value);
  if (/^[A-Za-z0-9_./:=@+%\\-]+$/.test(text)) {
    return text;
  }
  return `"${text.replace(/"/g, "\\\"")}"`;
}

function createLogPaths(): string[] {
  const timestamp = new Date().toISOString().replace(/[:]/g, "-");
  return [
    path.join(logDir, "latest.log"),
    path.join(logDir, `${timestamp}.log`)
  ];
}

function initializeLogs(logPaths: readonly string[], profile: Profile, totalChecks: number, leafChecks: number): void {
  fs.mkdirSync(logDir, { recursive: true });
  for (const logPath of logPaths) {
    fs.writeFileSync(
      logPath,
      [
        "docnav workspace verification",
        `started: ${new Date().toISOString()}`,
        `cwd: ${root}`,
        `profile: ${profile}`,
        `checks: ${totalChecks}`,
        `leaf checks: ${leafChecks}`,
        ""
      ].join("\n"),
      "utf8"
    );
  }
}

function finalizeLogs(logPaths: readonly string[], totalDurationMs: number): void {
  for (const logPath of logPaths) {
    fs.appendFileSync(logPath, `completed: ${new Date().toISOString()}\n`, "utf8");
    fs.appendFileSync(logPath, `duration: ${formatDurationMs(totalDurationMs)}\n`, "utf8");
  }
}

function printUsage(writeLine: (line: string) => void): void {
  writeLine("Usage: node scripts/docnav-workspace/verify.ts [--profile required|full] [--concurrency <n>]");
  writeLine("");
  writeLine("Profiles:");
  for (const [name, profile] of Object.entries(profiles)) {
    writeLine(`  - ${name}: ${profile.description}`);
  }
}

function relativeLogPath(logPath: string): string {
  return toSlashPath(path.relative(root, logPath));
}

function isMainModule() {
  return process.argv[1] ? path.resolve(process.argv[1]) === fileURLToPath(import.meta.url) : false;
}
