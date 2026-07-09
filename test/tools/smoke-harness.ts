import { AsyncLocalStorage } from "node:async_hooks";
import fs from "node:fs";
import path from "node:path";
import type { ValidateFunction } from "ajv";

import { runParallelTasks } from "../../scripts/tools/parallel-task-runner/src/index.ts";
import { errorMessage } from "../../scripts/tools/foundation/src/errors.ts";
import {
  compileRegisteredSchema,
  createSchemaAjv,
  formatAjvErrors
} from "../../scripts/tools/validators/schema/registry.ts";
import {
  createProcessOptions,
  normalizeProcessResult,
  spawnCommand,
  summarizeCommandStdin
} from "./smoke-harness/process.ts";
import type {
  PreparedCliCommand,
  ProcessResult,
  SmokeCommandOptions
} from "./smoke-harness/process.ts";
import {
  aggregateSmokeReports,
  prepareSmokeTasks,
  resolveSmokeConcurrency,
  withSmokeTaskMetadata
} from "./smoke-harness/tasks.ts";
import type {
  SmokeTask,
  SmokeTaskOptions,
  SmokeTestResult
} from "./smoke-harness/tasks.ts";
import {
  createSmokeAuditLog,
  formatAssertions
} from "./smoke-harness/audit-log.ts";

export type { SmokeCommandOptions } from "./smoke-harness/process.ts";
export {
  prepareSmokeTasks,
  resolveSmokeConcurrency
} from "./smoke-harness/tasks.ts";
export type {
  SmokeTask,
  SmokeTestResult
} from "./smoke-harness/tasks.ts";

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
  const auditLog = createSmokeAuditLog({
    auditMetadata,
    auditTitle,
    binaryFallback,
    binaryPath,
    logDir,
    logPaths,
    root,
    safeArgPattern,
    state
  });

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
    const command = prepareCliCommand(commandOptions);
    const result = await runProcess(command.executable, args, command.processOptions);
    const record = createCommandRecord(name, args, command, commandOptions, result);

    recordCommand(state, record);
    if (record.error) {
      expect(record, false, `process spawned successfully: ${record.error}`);
    }
    return record;
  }

  function prepareCliCommand(commandOptions: SmokeCommandOptions): PreparedCliCommand {
    const cwd = resolveCwd(commandOptions);
    const executable = binaryPath();
    if (!executable) {
      throw new Error("smoke binary path is not configured");
    }

    return {
      cwd,
      executable,
      processOptions: createProcessOptions(commandOptions, cwd, resolveEnv(commandOptions))
    };
  }

  function createCommandRecord(
    name: string,
    args: string[],
    command: PreparedCliCommand,
    commandOptions: SmokeCommandOptions,
    result: ProcessResult
  ): CommandRecord {
    return {
      name,
      args,
      cwd: command.cwd,
      stdinSummary: summarizeCommandStdin(commandOptions),
      ...normalizeProcessResult(result),
      assertions: []
    };
  }

  async function runSmokeTasks(tasks: readonly SmokeTask[], taskOptions: SmokeTaskOptions = {}) {
    const concurrencyValue = taskOptions.concurrency === undefined
      ? process.env.DOCNAV_SMOKE_CONCURRENCY
      : taskOptions.concurrency;
    const results = await runParallelTasks(tasks, {
      concurrency: resolveSmokeConcurrency(concurrencyValue),
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

  function printLogLocation(writeLine: (line: string) => void) {
    writeLine("");
    writeLine("Log:");
    writeLine(`  - ${relativeLogPath(root, logPaths[0])}`);
    writeLine("");
  }

  return {
    compileSchemas,
    formatAssertions,
    formatCommandRecord: auditLog.formatCommandRecord,
    printFailureSummary,
    printSuccessSummary,
    runCli,
    runSmokeTasks,
    runTest,
    validateSchema,
    writeAuditLogs: auditLog.writeAuditLogs
  };
}

function recordCommand(state: SmokeState, record: CommandRecord) {
  state.commandRecords.push(record);
  const context = commandContext.getStore();
  context?.commandRecords.push(record);
}

function relativeLogPath(root: string, logPath: string): string {
  return path.relative(root, logPath).replaceAll(path.sep, "/");
}
