import { Buffer } from "node:buffer";
import { spawn } from "node:child_process";
import { AsyncLocalStorage } from "node:async_hooks";
import fs from "node:fs";
import path from "node:path";
import type { ValidateFunction } from "ajv";

import { expandTasks, runParallelTasks } from "../../scripts/tools/parallel-task-runner/index.ts";
import type { NormalizedTask, TaskDefinition } from "../../scripts/tools/parallel-task-runner/index.ts";
import { errorMessage } from "../../scripts/tools/errors.ts";
import {
  compileRegisteredSchema,
  createSchemaAjv,
  formatAjvErrors
} from "../../scripts/tools/validators/schema/registry.ts";

const MAX_COMMAND_OUTPUT = 1024 * 1024 * 64;
const commandContext = new AsyncLocalStorage<{ commandRecords: CommandRecord[] }>();

export interface AssertionRecord {
  ok: boolean;
  summary: string;
}

export interface CommandRecord {
  args: string[];
  assertions: AssertionRecord[];
  cwd: string;
  error: string | null;
  exitCode: number;
  name: string;
  signal: NodeJS.Signals | null;
  stderr: string;
  stdinSummary: string | null;
  stdout: string;
}

export interface SmokeState {
  binaryPath?: string | null;
  commandRecords: CommandRecord[];
  docnavBinaryPath?: string | null;
  markdownBinaryPath?: string | null;
  normalRef?: string | null;
  normalRefPromise?: Promise<string> | null;
  startedAt: Date;
  testResults: SmokeTestResult[];
  validators: Record<string, ValidateFunction> | null;
  [key: string]: unknown;
}

export interface SmokeTestResult {
  commandCount: number;
  durationMs: number;
  endedAtMs: number;
  error?: Error;
  id: string;
  label: string;
  ok: boolean;
  reportId?: string;
  reportLabel?: string;
  reportOrder?: number;
  startedAtMs: number;
}

export interface SmokeTask extends TaskDefinition {
  label: string;
  reportId?: string;
  reportLabel?: string;
  reportOrder?: number;
  run?: (task: NormalizedTask) => unknown | Promise<unknown>;
}

export interface SmokeCommandOptions {
  cwd?: string;
  env?: NodeJS.ProcessEnv;
  maxBuffer?: number;
  project?: { env?: NodeJS.ProcessEnv; root?: string };
  stdin?: Buffer | string | null;
  stdinSummary?: string | null;
}

interface ProcessResult {
  error: string | null;
  exitCode: number;
  signal: NodeJS.Signals | null;
  stderr: string;
  stdout: string;
}

interface CreateSmokeHarnessOptions {
  auditMetadata: () => string[];
  auditTitle: string;
  binaryFallback: string;
  binaryPath: () => string | null;
  expect: (record: CommandRecord, condition: boolean, summary: string) => void;
  logDir: string;
  logPaths: string[];
  resolveCwd?: (options: SmokeCommandOptions) => string;
  resolveEnv?: (options: SmokeCommandOptions) => NodeJS.ProcessEnv | undefined;
  root: string;
  runProcess?: (executable: string, args: string[], options: SmokeCommandOptions) => Promise<ProcessResult>;
  safeArgPattern?: RegExp;
  schemaPaths: Record<string, string>;
  state: SmokeState;
  title: string;
}

interface SmokeTaskOptions {
  concurrency?: string | number | null;
}

export function createSmokeState(values: Partial<SmokeState> = {}): SmokeState {
  return {
    commandRecords: [],
    testResults: [],
    startedAt: new Date(),
    validators: null,
    ...values
  };
}

export function argValue(flag: string): string | null {
  const index = process.argv.indexOf(flag);
  return index === -1 ? null : (process.argv[index + 1] ?? null);
}

export function resolveBinaryPath(root: string, value: string | null | undefined): string | null {
  if (!value) {
    return null;
  }

  const resolved = path.resolve(root, value);
  if (fs.existsSync(resolved)) {
    return resolved;
  }
  if (process.platform === "win32" && !resolved.toLowerCase().endsWith(".exe")) {
    const executablePath = `${resolved}.exe`;
    if (fs.existsSync(executablePath)) {
      return executablePath;
    }
  }
  return resolved;
}

export function createSmokeHarness(options: CreateSmokeHarnessOptions) {
  const {
    state,
    root,
    logDir,
    logPaths,
    schemaPaths,
    expect,
    title,
    auditTitle,
    auditMetadata,
    binaryPath,
    binaryFallback,
    resolveCwd = () => root,
    resolveEnv = () => undefined,
    runProcess = spawnCommand,
    safeArgPattern = /^[A-Za-z0-9_./:=@+-]+$/
  } = options;

  async function runTest(label: string, fn: () => unknown | Promise<unknown>, options: { id?: string } = {}) {
    const context = { commandRecords: [] as CommandRecord[] };
    const startedAtMs = Date.now();
    const result: SmokeTestResult = {
      id: options.id ?? label,
      label,
      ok: true,
      commandCount: 0,
      durationMs: 0,
      startedAtMs,
      endedAtMs: startedAtMs
    };
    try {
      await commandContext.run(context, () => fn());
    } catch (error) {
      result.ok = false;
      result.error = error instanceof Error ? error : new Error(String(error));
    }
    result.commandCount = context.commandRecords.length;
    result.endedAtMs = Date.now();
    result.durationMs = result.endedAtMs - startedAtMs;
    return result;
  }

  async function runCli(name: string, args: string[], commandOptions: SmokeCommandOptions = {}) {
    const cwd = resolveCwd(commandOptions);
    const executable = binaryPath();
    if (!executable) {
      throw new Error("smoke binary path is not configured");
    }
    const result = await runProcess(executable, args, {
      cwd,
      env: resolveEnv(commandOptions),
      stdin: commandOptions.stdin,
      maxBuffer: MAX_COMMAND_OUTPUT
    });

    const record = {
      name,
      args,
      cwd,
      stdinSummary: commandOptions.stdinSummary ?? summarizeStdin(commandOptions.stdin),
      exitCode: result.exitCode ?? 1,
      signal: result.signal ?? null,
      error: result.error ?? null,
      stdout: result.stdout ?? "",
      stderr: result.stderr ?? "",
      assertions: []
    };
    recordCommand(state, record);
    if (record.error) {
      expect(record, false, `process spawned successfully: ${record.error}`);
    }
    return record;
  }

  async function runSmokeTasks(tasks: readonly SmokeTask[], taskOptions: SmokeTaskOptions = {}) {
    const results = await runParallelTasks(tasks, {
      concurrency: resolveSmokeConcurrency(taskOptions.concurrency),
      prepareTasks: (taskList) => prepareSmokeTasks(taskList as readonly SmokeTask[]),
      execute: async (task) => withSmokeTaskMetadata(await runTest(task.label, () => task.run?.(task), { id: task.id }), task)
    });
    const reports = aggregateSmokeReports(results);
    state.testResults.push(...reports);
    return reports;
  }

  function compileSchemas() {
    const ajv = createSchemaAjv();
    return Object.fromEntries(
      Object.entries(schemaPaths).map(([name, relativePath]) => [name, compileRegisteredSchema(ajv, relativePath)])
    );
  }

  function validateSchema(record: CommandRecord, name: string, value: unknown) {
    const validate = state.validators?.[name];
    expect(record, Boolean(validate), `schema validator exists for ${name}`);
    if (!validate) {
      return;
    }
    const ok = validate(value);
    const details = ok ? "" : `: ${formatAjvErrors(validate)}`;
    expect(record, ok, `${name} schema valid${details}`);
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

  function printSuccessSummary() {
    console.log("");
    console.log(title);
    console.log("Status: passed");
    console.log(`Commands: ${state.commandRecords.length}`);
    printLogLocation(console.log);
  }

  function printFailureSummary(error: unknown) {
    console.error("");
    console.error(title);
    console.error("Status: failed");
    console.error(`Failure: ${errorMessage(error)}`);
    printLogLocation(console.error);
  }

  function quoteArg(value: string) {
    return safeArgPattern.test(value) ? value : JSON.stringify(value);
  }

  function printLogLocation(writeLine: (line: string) => void) {
    writeLine("");
    writeLine("Log:");
    writeLine(`  - ${relativeLogPath(root, logPaths[0])}`);
    writeLine("");
  }

  return {
    compileSchemas,
    formatAssertions,
    formatCommandRecord,
    printFailureSummary,
    printSuccessSummary,
    runCli,
    runSmokeTasks,
    runTest,
    validateSchema,
    writeAuditLogs
  };
}

export function prepareSmokeTasks(tasks: readonly SmokeTask[]): NormalizedTask[] {
  return withSmokeReportMetadata(tasks);
}

export function resolveSmokeConcurrency(value: string | number | null | undefined = process.env.DOCNAV_SMOKE_CONCURRENCY): number | undefined {
  if (value === undefined || value === null || value === "") {
    return undefined;
  }
  const parsed = Number.parseInt(String(value), 10);
  if (!Number.isFinite(parsed) || parsed < 1 || String(parsed) !== String(value)) {
    throw new Error(`DOCNAV_SMOKE_CONCURRENCY must be a positive integer: ${value}`);
  }
  return parsed;
}

function withSmokeReportMetadata(tasks: readonly SmokeTask[]): NormalizedTask[] {
  let reportOrder = 0;
  return expandTasks(tasks.map((task) => annotateSmokeReport(task, null, () => reportOrder++)));
}

function annotateSmokeReport(
  task: SmokeTask,
  inheritedReport: { id: string; label: string; order: number } | null,
  nextReportOrder: () => number
): SmokeTask {
  const report = inheritedReport ?? createSmokeReport(task, nextReportOrder);
  if (Array.isArray(task.tasks)) {
    return {
      ...task,
      tasks: task.tasks.map((child) => annotateSmokeReport(child as SmokeTask, report, nextReportOrder))
    };
  }
  return {
    ...task,
    reportId: report.id,
    reportLabel: report.label,
    reportOrder: report.order
  };
}

function createSmokeReport(task: SmokeTask, nextReportOrder: () => number) {
  return {
    id: task.id,
    label: task.label ?? task.id,
    order: nextReportOrder()
  };
}

function withSmokeTaskMetadata(result: SmokeTestResult, task: NormalizedTask): SmokeTestResult {
  return {
    ...result,
    reportId: task.reportId as string | undefined,
    reportLabel: task.reportLabel as string | undefined,
    reportOrder: task.reportOrder as number | undefined
  };
}

function recordCommand(state: SmokeState, record: CommandRecord) {
  state.commandRecords.push(record);
  const context = commandContext.getStore();
  context?.commandRecords.push(record);
}

function spawnCommand(executable: string, args: string[], options: SmokeCommandOptions): Promise<ProcessResult> {
  return new Promise((resolve) => {
    let childError: Error | null = null;
    let settled = false;
    const maxBuffer = options.maxBuffer ?? MAX_COMMAND_OUTPUT;

    const child = spawn(executable, args, {
      cwd: options.cwd,
      env: options.env,
      windowsHide: true,
      stdio: "pipe"
    });

    const output = createCommandOutputCapture(maxBuffer, () => {
      if (!child.killed) {
        childError = new Error(`command output exceeded ${maxBuffer} bytes`);
        child.kill();
      }
    });

    child.stdout.on("data", (chunk: unknown) => output.append(chunk, "stdout"));
    child.stderr.on("data", (chunk: unknown) => output.append(chunk, "stderr"));
    child.on("error", (error: Error) => {
      childError = error;
      finish(1, null);
    });
    child.on("close", (exitCode, signal) => finish(exitCode, signal));
    child.stdin.on("error", () => {});

    if (options.stdin !== undefined && options.stdin !== null) {
      child.stdin.end(options.stdin);
    } else {
      child.stdin.end();
    }

    function finish(exitCode: number | null, signal: NodeJS.Signals | null) {
      if (settled) {
        return;
      }
      settled = true;
      const { stdout, stderr } = output.snapshot();
      resolve({
        exitCode: exitCode ?? 1,
        signal,
        error: childError?.message ?? null,
        stdout,
        stderr
      });
    }
  });
}

interface CommandOutputCapture {
  append: (chunk: unknown, streamName: "stderr" | "stdout") => void;
  snapshot: () => Pick<ProcessResult, "stderr" | "stdout">;
}

function createCommandOutputCapture(maxBuffer: number, onMaxBufferExceeded: () => void): CommandOutputCapture {
  let stdout = "";
  let stderr = "";
  let stdoutBytes = 0;
  let stderrBytes = 0;
  let maxBufferExceeded = false;

  return {
    append(chunk, streamName) {
      const text = commandOutputText(chunk);
      const bytes = Buffer.byteLength(text, "utf8");
      if (streamName === "stdout") {
        stdout += text;
        stdoutBytes += bytes;
      } else {
        stderr += text;
        stderrBytes += bytes;
      }
      if (stdoutBytes + stderrBytes > maxBuffer && !maxBufferExceeded) {
        maxBufferExceeded = true;
        onMaxBufferExceeded();
      }
    },
    snapshot() {
      return { stdout, stderr };
    }
  };
}

function commandOutputText(chunk: unknown): string {
  if (Buffer.isBuffer(chunk)) {
    return chunk.toString("utf8");
  }
  if (typeof chunk === "string") {
    return chunk;
  }
  return String(chunk);
}

function formatTestResult(result: SmokeTestResult) {
  const status = result.ok ? "PASS" : "FAIL";
  const error = result.error ? `: ${result.error.message}` : "";
  const duration = Number.isFinite(result.durationMs) ? `, ${result.durationMs}ms` : "";
  return `${status} ${result.label} (${result.commandCount} command(s)${duration})${error}`;
}

function aggregateSmokeReports(results: readonly SmokeTestResult[]): SmokeTestResult[] {
  const reports = new Map<string, SmokeTestResult>();

  for (const result of results) {
    const report = getSmokeReport(reports, result);
    mergeSmokeReportResult(report, result);
  }

  return sortSmokeReports(reports);
}

function getSmokeReport(reports: Map<string, SmokeTestResult>, result: SmokeTestResult): SmokeTestResult {
  const reportId = result.reportId ?? result.id ?? result.label;
  const report = reports.get(reportId);
  if (report) {
    return report;
  }

  const createdReport = createSmokeReportResult(result, reportId, reports.size);
  reports.set(reportId, createdReport);
  return createdReport;
}

function createSmokeReportResult(result: SmokeTestResult, reportId: string, reportOrder: number): SmokeTestResult {
  const reportLabel = result.reportLabel ?? result.label;
  return {
    id: reportId,
    label: reportLabel,
    reportId,
    reportLabel,
    reportOrder: result.reportOrder ?? reportOrder,
    ok: true,
    commandCount: 0,
    durationMs: 0,
    startedAtMs: result.startedAtMs,
    endedAtMs: result.endedAtMs
  };
}

function mergeSmokeReportResult(report: SmokeTestResult, result: SmokeTestResult) {
  report.ok &&= result.ok;
  report.commandCount += result.commandCount;
  report.startedAtMs = Math.min(report.startedAtMs, result.startedAtMs);
  report.endedAtMs = Math.max(report.endedAtMs, result.endedAtMs);
  report.durationMs = report.endedAtMs - report.startedAtMs;
  if (!result.ok && !report.error) {
    report.error = result.error;
  }
}

function sortSmokeReports(reports: Map<string, SmokeTestResult>): SmokeTestResult[] {
  return [...reports.values()].sort((left, right) => (left.reportOrder ?? 0) - (right.reportOrder ?? 0));
}

function formatAssertions(assertions: readonly AssertionRecord[]): string[] {
  if (assertions.length === 0) {
    return ["  (none)"];
  }
  return assertions.map((assertion) => `  ${assertion.ok ? "PASS" : "FAIL"} ${assertion.summary}`);
}

function summarizeStdin(stdin: Buffer | string | null | undefined): string | null {
  if (stdin === undefined || stdin === null) {
    return null;
  }
  const byteCount = Buffer.isBuffer(stdin) ? stdin.length : Buffer.byteLength(String(stdin), "utf8");
  const unit = byteCount === 1 ? "byte" : "bytes";
  return `${byteCount} ${unit} stdin`;
}

function relativeLogPath(root: string, logPath: string): string {
  return path.relative(root, logPath).replaceAll(path.sep, "/");
}
