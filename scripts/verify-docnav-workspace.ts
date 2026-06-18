import { execFile } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

import { expandTasks, runParallelTasks } from "./tools/parallel-task-runner.ts";
import type { NormalizedTask, TaskDefinition } from "./tools/parallel-task-runner.ts";
import { errorMessage, isRecord, isStringArray, isUnknownArray, processFailure } from "./tools/types.ts";
import type { ProcessFailure, StringMap } from "./tools/types.ts";

const execFileAsync = promisify(execFile);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const logDir = path.join(root, ".log", "verify-docnav-workspace");
const MAX_BUFFER = 1024 * 1024 * 64;
const DEV_BIN_ENV_FILE = ".log/verify-docnav-workspace/dev-bins.json";

export const PROFILE_REQUIRED = "required";
export const PROFILE_FULL = "full";

type Profile = typeof PROFILE_REQUIRED | typeof PROFILE_FULL;

type CheckDefinition = TaskDefinition & {
  args?: string[];
  command?: string;
  ignoreOutput?: RegExp[];
  tasks?: readonly CheckDefinition[];
};

interface CheckTask extends NormalizedTask {
  args: string[];
  command: string;
  ignoreOutput: RegExp[];
  reportId?: string;
  reportLabel?: string;
}

interface ParsedVerificationOptions {
  help: boolean;
  profile: Profile;
  concurrency: string | number | undefined;
}

interface VerificationOptions {
  help: boolean;
  profile: Profile;
  concurrency: number | undefined;
}

interface CheckReportRef {
  id: string;
  label: string;
}

interface CompletionResult {
  check: CheckReportRef;
  combinedOutput: string;
  durationMs: number;
  endedAtMs: number;
  error: ProcessFailure | null;
  exitCode: number;
  ok: boolean;
  startedAtMs: number;
  stderr: string;
  stdout: string;
}

interface CheckResult extends CompletionResult {
  check: CheckTask;
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

interface ReportAccumulator {
  check: CheckReportRef;
  completed: number;
  endedAtMs: number;
  error: ProcessFailure | null;
  exitCode: number;
  expected: number;
  ok: boolean;
  startedAtMs: number;
}

export const profiles = Object.freeze({
  [PROFILE_REQUIRED]: {
    label: "required",
    description: "fast deterministic checks for routine development"
  },
  [PROFILE_FULL]: {
    label: "full",
    description: "required checks plus smoke, Rust, and OpenSpec gates"
  }
});

const nodeTestSuccessOutput = [
  /^TAP version \d+$/,
  /^\s*▶ /,
  /^\s*✔ /,
  /^\s*ℹ /,
  /^# Subtest:/,
  /^ok \d+ -/,
  /^1\.\.\d+$/,
  /^# (tests|suites|pass|fail|cancelled|skipped|todo|duration_ms) /
];

const cargoFinishedOutput = [
  /^\s*Finished `.*` profile .*$/
];

export const checks = defineChecks([
  {
    id: "required-checks",
    type: PROFILE_REQUIRED,
    tasks: [
      {
        id: "cargo-fmt",
        label: "cargo fmt",
        command: "cargo",
        args: ["fmt", "--all", "--check"]
      },
      {
        id: "typecheck-scripts",
        label: "TypeScript script typecheck",
        command: "pnpm",
        args: ["run", "typecheck:scripts"],
        ignoreOutput: [
          /^\$ tsc -p tsconfig\.scripts\.json$/
        ]
      },
      {
        id: "lint-scripts",
        label: "TypeScript script lint",
        command: "pnpm",
        args: ["run", "lint:scripts"],
        ignoreOutput: [
          /^\$ eslint --max-warnings 0 eslint\.config\.ts scripts\/\*\*\/\*\.ts test\/\*\*\/\*\.ts$/
        ]
      },
      {
        id: "generated-error-rules",
        label: "generated error rules",
        command: "node",
        args: ["scripts/generate-error-rules.ts", "--check"],
        ignoreOutput: [
          /^generated error rules ok$/
        ]
      },
      {
        id: "docs-validators",
        label: "docs validators",
        tasks: docsValidatorChecks()
      },
      {
        id: "workspace-verifier-script-tests",
        label: "workspace verifier script tests",
        tasks: nodeTestFileChecks([
          ["workspace-verifier-tests", "workspace verifier tests", "scripts/tools/verify-docnav-workspace.test.ts"],
          ["smoke-harness-tests", "smoke harness tests", "test/tools/smoke-harness.test.ts"],
          ["parallel-task-runner-tests", "parallel task runner tests", "scripts/tools/parallel-task-runner.test.ts"]
        ])
      },
      {
        id: "release-package-script-tests",
        label: "release package script tests",
        command: "node",
        args: ["--test", "scripts/tools/release-package/args.test.ts"],
        ignoreOutput: [
          ...nodeTestSuccessOutput
        ]
      },
      {
        id: "git-diff-whitespace",
        label: "git diff whitespace",
        command: "git",
        args: ["diff", "--check"],
        ignoreOutput: [
          /LF will be replaced by CRLF/i
        ]
      }
    ]
  },
  {
    id: "full-checks",
    type: PROFILE_FULL,
    tasks: [
      {
        id: "quality-tool-tests",
        label: "quality tool tests",
        tasks: nodeTestFileChecks([
          ["quality-tools-tests", "quality tools tests", "scripts/tools/quality/tools.test.ts"]
        ])
      },
      {
        id: "docnav-development-smoke",
        label: "docnav development smoke",
        tasks: [
          {
            id: "docnav-development-binaries",
            label: "docnav development binaries",
            command: "node",
            args: ["scripts/build-docnav-dev-bins.ts", "--quiet", "--output-env-json", DEV_BIN_ENV_FILE],
            mutex: ["cargo-build"],
            ignoreOutput: [
              /^dev binaries ok: DOCNAV_BIN, DOCNAV_MARKDOWN_BIN$/
            ]
          },
          {
            id: "docnav-development-smoke-execution",
            dependsOn: ["docnav-development-binaries"],
            envFile: DEV_BIN_ENV_FILE,
            tasks: [
              {
                id: "docnav-markdown-development-smoke",
                label: "docnav-markdown development smoke",
                command: "node",
                args: ["test/docnav-markdown-smoke.ts"],
                ignoreOutput: [
                  ...smokeSuccessOutput("Docnav Markdown Development Smoke", ".log/docnav-markdown-cli-smoke/latest.log")
                ]
              },
              {
                id: "docnav-core-development-smoke",
                label: "docnav core development smoke",
                command: "node",
                args: ["test/docnav-core-smoke.ts"],
                ignoreOutput: [
                  ...smokeSuccessOutput("Docnav Core Development Smoke", ".log/docnav-core-cli-smoke/latest.log")
                ]
              }
            ]
          }
        ]
      },
      {
        id: "cargo-clippy",
        label: "cargo clippy",
        command: "cargo",
        args: ["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
        mutex: ["cargo-build"],
        ignoreOutput: [
          ...cargoFinishedOutput
        ]
      },
      {
        id: "cargo-test",
        label: "cargo test",
        command: "cargo",
        args: ["test", "--workspace"],
        mutex: ["cargo-build"],
        ignoreOutput: [
          ...cargoFinishedOutput,
          /^\s*Running unittests .*$/,
          /^\s*Running tests[\\/].*$/,
          /^\s*Doc-tests .*$/,
          /^running \d+ tests$/,
          /^test .* \.\.\. ok$/,
          /^test result: ok\..*$/
        ]
      },
      {
        id: "openspec",
        label: "openspec",
        command: "openspec",
        args: ["validate", "--all", "--strict"],
        ignoreOutput: [
          /^✓ /,
          /^Totals: \d+ passed, 0 failed .*$/,
          /^- Validating\.\.\.$/
        ]
      }
    ]
  }
]);

if (isMainModule()) {
  void main();
}

export function parseArgs(argv: string[]): VerificationOptions {
  const options: ParsedVerificationOptions = {
    help: false,
    profile: PROFILE_FULL,
    concurrency: undefined
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--help" || arg === "-h") {
      options.help = true;
      continue;
    }
    if (arg === "--profile") {
      const value = argv[index + 1];
      if (!value) {
        throw new Error("--profile requires a value");
      }
      options.profile = parseProfile(value);
      index += 1;
      continue;
    }
    if (arg.startsWith("--profile=")) {
      options.profile = parseProfile(arg.slice("--profile=".length));
      continue;
    }
    if (arg === "--concurrency") {
      const value = argv[index + 1];
      if (!value) {
        throw new Error("--concurrency requires a value");
      }
      options.concurrency = value;
      index += 1;
      continue;
    }
    if (arg.startsWith("--concurrency=")) {
      options.concurrency = arg.slice("--concurrency=".length);
      continue;
    }
    throw new Error(`unknown argument: ${arg}`);
  }

  return {
    ...options,
    concurrency: resolveVerificationConcurrency(options.concurrency?.toString())
  };
}

export function checksForProfile(profile: Profile): CheckTask[] {
  assertProfile(profile);
  if (profile === PROFILE_REQUIRED) {
    return checks.filter((check) => check.type === PROFILE_REQUIRED);
  }
  return checks.filter((check) => check.type === PROFILE_REQUIRED || check.type === PROFILE_FULL);
}

export function visibleOutputLines(check: CheckTask, output: string): string[] {
  return lines(output).filter((line) => !isIgnoredOutput(check, line));
}

export function isIgnoredOutput(check: Pick<CheckTask, "ignoreOutput">, line: string): boolean {
  return (check.ignoreOutput ?? []).some((pattern) => pattern.test(line));
}

export function formatCompletionLine(result: Pick<CompletionResult, "check" | "durationMs" | "ok">): string {
  return `  ${result.ok ? "passed" : "failed"}: ${result.check.label} (${formatDurationMs(result.durationMs)})`;
}

export function reportCountForChecks(checkList: readonly CheckTask[]): number {
  return new Set(checkList.map(reportIdForCheck)).size;
}

export function formatDurationMs(durationMs: number): string {
  if (!Number.isFinite(durationMs)) {
    return "unknown";
  }
  if (durationMs < 1000) {
    return `${Math.max(0, Math.round(durationMs))}ms`;
  }
  const totalSeconds = durationMs / 1000;
  if (totalSeconds < 60) {
    return `${totalSeconds.toFixed(totalSeconds < 10 ? 1 : 0)}s`;
  }
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = Math.round(totalSeconds % 60).toString().padStart(2, "0");
  return `${minutes}m ${seconds}s`;
}

export function resolveVerificationConcurrency(value = process.env.DOCNAV_VERIFY_CONCURRENCY) {
  if (value === undefined || value === null || value === "") {
    return undefined;
  }
  const parsed = Number.parseInt(String(value), 10);
  if (!Number.isFinite(parsed) || parsed < 1 || String(parsed) !== String(value)) {
    throw new Error(`verification concurrency must be a positive integer: ${value}`);
  }
  return parsed;
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
  const invocation = commandInvocation(check);
  try {
    const { stdout, stderr } = await execFileAsync(invocation.command, invocation.args, {
      cwd: root,
      env: environmentForCheck(check),
      encoding: "utf8",
      windowsHide: true,
      maxBuffer: MAX_BUFFER
    });
    return buildCheckResult(check, {
      ok: true,
      exitCode: 0,
      stdout: stdout ?? "",
      stderr: stderr ?? "",
      startedAtMs,
      endedAtMs: Date.now()
    });
  } catch (error: unknown) {
    const failure = processFailure(error);
    return buildCheckResult(check, {
      ok: false,
      exitCode: normalizeExitCode(failure),
      stdout: typeof failure.stdout === "string" ? failure.stdout : "",
      stderr: typeof failure.stderr === "string" ? failure.stderr : "",
      error: failure,
      startedAtMs,
      endedAtMs: Date.now()
    });
  }
}

function buildCheckResult(check: CheckTask, data: CheckExecutionData): CheckResult {
  const combinedOutput = [data.stdout, data.stderr].filter(Boolean).join("\n");
  return {
    check,
    ok: data.ok,
    exitCode: data.exitCode,
    error: data.error === undefined ? null : processFailure(data.error),
    stdout: data.stdout,
    stderr: data.stderr,
    combinedOutput,
    durationMs: data.endedAtMs - data.startedAtMs,
    startedAtMs: data.startedAtMs,
    endedAtMs: data.endedAtMs
  };
}

function normalizeExitCode(error: ProcessFailure): number {
  return typeof error?.code === "number" ? error.code : 1;
}

function commandInvocation(check: CheckTask): { command: string; args: string[] } {
  if (process.platform !== "win32") {
    return check;
  }

  return {
    command: process.env.ComSpec || "cmd.exe",
    args: ["/d", "/s", "/c", commandLine(check)]
  };
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
  const parsed: unknown = JSON.parse(fs.readFileSync(envPath, "utf8"));
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
}: {
  profile: Profile;
  totalChecks: number;
  completedResults: readonly CompletionResult[];
  totalDurationMs: number;
  logPaths: readonly string[];
}): void {
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
    `$ ${commandLine(result.check)}`,
    `exit: ${result.exitCode}`,
    `duration: ${formatDurationMs(result.durationMs)}`,
    result.error ? `spawn_error: ${result.error.message}` : null,
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

function defineChecks(checkList: readonly CheckDefinition[]): CheckTask[] {
  return withCheckReportMetadata(checkList).map(asCheckTask);
}

function withCheckReportMetadata(checkList: readonly CheckDefinition[]): NormalizedTask[] {
  return expandTasks(checkList.map((check) => annotateCheckReport(check, null)));
}

function annotateCheckReport(check: CheckDefinition, inheritedReport: CheckReportRef | null): CheckDefinition {
  const report = inheritedReport ?? (typeof check.label === "string" ? createCheckReport(check) : null);
  const childChecks = check.tasks;
  if (childChecks !== undefined) {
    const maybeChildChecks: unknown = childChecks;
    if (!Array.isArray(maybeChildChecks)) {
      throw new Error(`check ${check.id} tasks must be an array`);
    }
    return {
      ...check,
      tasks: childChecks.map((child) => annotateCheckReport(child, report))
    };
  }
  const leafReport = report ?? createCheckReport(check);
  return {
    ...check,
    reportId: leafReport.id,
    reportLabel: leafReport.label
  };
}

function createCheckReport(check: CheckDefinition): CheckReportRef {
  return {
    id: check.id,
    label: check.label ?? check.id
  };
}

function asCheckTask(task: NormalizedTask): CheckTask {
  const args = isStringArray(task.args) ? task.args : [];
  const command = typeof task.command === "string" ? task.command : "";
  const ignoreOutput = isRegExpArray(task.ignoreOutput) ? task.ignoreOutput : [];
  return {
    ...task,
    args,
    command,
    ignoreOutput
  };
}

function isRegExpArray(value: unknown): value is RegExp[] {
  return isUnknownArray(value) && value.every((item) => item instanceof RegExp);
}

function createReportCompletionTracker(checkList: readonly CheckTask[]): (result: CheckResult) => CompletionResult | null {
  const reports = new Map<string, ReportAccumulator>();
  for (const check of checkList) {
    const reportId = reportIdForCheck(check);
    const report = reports.get(reportId) ?? {
      check: {
        id: reportId,
        label: reportLabelForCheck(check)
      },
      expected: 0,
      completed: 0,
      ok: true,
      exitCode: 0,
      error: null,
      startedAtMs: Number.POSITIVE_INFINITY,
      endedAtMs: 0
    };
    report.expected += 1;
    reports.set(reportId, report);
  }

  return (result: CheckResult) => {
    const report = reports.get(reportIdForCheck(result.check));
    if (!report) {
      throw new Error(`missing report for check: ${result.check.id}`);
    }
    report.completed += 1;
    report.ok &&= result.ok;
    report.startedAtMs = Math.min(report.startedAtMs, result.startedAtMs);
    report.endedAtMs = Math.max(report.endedAtMs, result.endedAtMs);
    if (!result.ok && !report.error) {
      report.error = result.error;
      report.exitCode = result.exitCode;
    }
    if (report.completed !== report.expected) {
      return null;
    }
    return {
      check: report.check,
      ok: report.ok,
      exitCode: report.exitCode,
      error: report.error,
      stdout: "",
      stderr: "",
      combinedOutput: "",
      durationMs: report.endedAtMs - report.startedAtMs,
      startedAtMs: report.startedAtMs,
      endedAtMs: report.endedAtMs
    };
  };
}

function reportIdForCheck(check: CheckTask): string {
  return check.reportId ?? check.id;
}

function reportLabelForCheck(check: CheckTask): string {
  return check.reportLabel ?? check.label;
}

function docsValidatorChecks(): CheckDefinition[] {
  return [
    {
      id: "docs-json-validator",
      label: "docs json validator",
      command: "node",
      args: ["scripts/validate-docs.ts", "json"],
      ignoreOutput: [
        /^json syntax ok:/
      ]
    },
    {
      id: "docs-schema-validator",
      label: "docs schema validator",
      command: "node",
      args: ["scripts/validate-docs.ts", "schema"],
      ignoreOutput: [
        /^schema strict compile ok:/,
        /^schema ok:/,
        /^protocol response operation\/result binding ok$/,
        /^protocol response error details requirements ok$/
      ]
    },
    {
      id: "docs-mcp-validator",
      label: "docs mcp validator",
      command: "node",
      args: ["scripts/validate-docs.ts", "mcp"],
      ignoreOutput: [
        /^mcp structuredContent ok:/
      ]
    },
    {
      id: "docs-example-consistency-validator",
      label: "docs example consistency validator",
      command: "node",
      args: ["scripts/validate-docs.ts", "examples"],
      ignoreOutput: [
        /^protocol\/readable mapping ok:/,
        /^error details ok:/,
        /^manifest example consistency ok:/,
        /^document output mode consistency ok:/
      ]
    },
    {
      id: "docs-links-validator",
      label: "docs links validator",
      command: "node",
      args: ["scripts/validate-docs.ts", "links"],
      ignoreOutput: [
        /^markdown links ok:/
      ]
    }
  ];
}

function nodeTestFileChecks(testFiles: readonly [id: string, label: string, filePath: string][]): CheckDefinition[] {
  return testFiles.map(([id, label, filePath]) => ({
    id,
    label,
    command: "node",
    args: ["--test", filePath],
    ignoreOutput: [
      ...nodeTestSuccessOutput
    ]
  }));
}

function smokeSuccessOutput(title: string, logPath: string): RegExp[] {
  return [
    new RegExp(`^${escapeRegex(title)}$`),
    /^Status: passed$/,
    /^Commands: \d+$/,
    /^Log:$/,
    new RegExp(`^\\s+- ${escapeRegex(logPath)}$`)
  ];
}

function commandLine(check: { args: readonly string[]; command: string }): string {
  return [check.command, ...check.args].map(quoteArg).join(" ");
}

function quoteArg(value: string): string {
  if (/^[A-Za-z0-9_./:=@+-]+$/.test(value)) {
    return value;
  }
  return JSON.stringify(value);
}

function lines(output: string): string[] {
  return output.split(/\r?\n/).filter((line) => line.length > 0);
}

function escapeRegex(value: string): string {
  return String(value).replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function parseProfile(profile: string): Profile {
  assertProfile(profile);
  return profile;
}

function assertProfile(profile: string): asserts profile is Profile {
  if (!Object.hasOwn(profiles, profile)) {
    throw new Error(`unknown verification profile: ${profile}`);
  }
}

function printUsage(writeLine: (line: string) => void): void {
  writeLine("Usage: node scripts/verify-docnav-workspace.ts [--profile required|full] [--concurrency <n>]");
  writeLine("");
  writeLine("Profiles:");
  for (const [name, profile] of Object.entries(profiles)) {
    writeLine(`  - ${name}: ${profile.description}`);
  }
}

function relativeLogPath(logPath: string): string {
  return path.relative(root, logPath).replaceAll(path.sep, "/");
}

function isMainModule() {
  return process.argv[1] ? path.resolve(process.argv[1]) === fileURLToPath(import.meta.url) : false;
}
