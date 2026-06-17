import { Buffer } from "node:buffer";
import { spawn } from "node:child_process";
import { AsyncLocalStorage } from "node:async_hooks";
import fs from "node:fs";
import path from "node:path";

import { expandTasks, runParallelTasks } from "../../scripts/tools/parallel-task-runner.ts";
import {
  compileRegisteredSchema,
  createSchemaAjv,
  formatAjvErrors
} from "../../scripts/tools/validators/schema-registry.ts";

const MAX_COMMAND_OUTPUT = 1024 * 1024 * 64;
const commandContext = new AsyncLocalStorage<{ commandRecords: any[] }>();

export function createSmokeState(values: any = {}) {
  return {
    commandRecords: [],
    testResults: [],
    startedAt: new Date(),
    validators: null,
    ...values
  };
}

export function argValue(flag: any) {
  const index = process.argv.indexOf(flag);
  return index === -1 ? null : (process.argv[index + 1] ?? null);
}

export function resolveBinaryPath(root: any, value: any) {
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

export function createSmokeHarness(options: any) {
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
    safeArgPattern = /^[A-Za-z0-9_./:=@+\-]+$/
  } = options;

  async function runTest(label: any, fn: any, options: any = {}) {
    const context = { commandRecords: [] as any[] };
    const startedAtMs = Date.now();
    const result: {
      id: any;
      label: any;
      ok: boolean;
      commandCount: number;
      durationMs: number;
      startedAtMs: number;
      endedAtMs: number;
      error?: any;
    } = {
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
      result.error = error;
    }
    result.commandCount = context.commandRecords.length;
    result.endedAtMs = Date.now();
    result.durationMs = result.endedAtMs - startedAtMs;
    return result;
  }

  async function runCli(name: any, args: any, commandOptions: any = {}) {
    const cwd = resolveCwd(commandOptions);
    const executable = binaryPath();
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

  async function runSmokeTasks(tasks: any, taskOptions: any = {}) {
    const results = await runParallelTasks(tasks, {
      concurrency: resolveSmokeConcurrency(taskOptions.concurrency),
      prepareTasks: prepareSmokeTasks,
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

  function validateSchema(record: any, name: any, value: any) {
    const validate = state.validators[name];
    expect(record, Boolean(validate), `schema validator exists for ${name}`);
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

  function formatCommandRecord(record: any) {
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

  function printFailureSummary(error: any) {
    console.error("");
    console.error(title);
    console.error("Status: failed");
    console.error(`Failure: ${error.message}`);
    printLogLocation(console.error);
  }

  function quoteArg(value: any) {
    return safeArgPattern.test(value) ? value : JSON.stringify(value);
  }

  function printLogLocation(writeLine: any) {
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

export function prepareSmokeTasks(tasks: any) {
  return withSmokeReportMetadata(tasks);
}

export function resolveSmokeConcurrency(value = process.env.DOCNAV_SMOKE_CONCURRENCY) {
  if (value === undefined || value === null || value === "") {
    return undefined;
  }
  const parsed = Number.parseInt(String(value), 10);
  if (!Number.isFinite(parsed) || parsed < 1 || String(parsed) !== String(value)) {
    throw new Error(`DOCNAV_SMOKE_CONCURRENCY must be a positive integer: ${value}`);
  }
  return parsed;
}

function withSmokeReportMetadata(tasks: any) {
  let reportOrder = 0;
  return expandTasks(tasks.map((task: any) => annotateSmokeReport(task, null, () => reportOrder++)));
}

function annotateSmokeReport(task: any, inheritedReport: any, nextReportOrder: any) {
  const report = inheritedReport ?? createSmokeReport(task, nextReportOrder);
  if (Array.isArray(task.tasks)) {
    return {
      ...task,
      tasks: task.tasks.map((child: any) => annotateSmokeReport(child, report, nextReportOrder))
    };
  }
  return {
    ...task,
    reportId: report.id,
    reportLabel: report.label,
    reportOrder: report.order
  };
}

function createSmokeReport(task: any, nextReportOrder: any) {
  return {
    id: task.id,
    label: task.label ?? task.id,
    order: nextReportOrder()
  };
}

function withSmokeTaskMetadata(result: any, task: any) {
  return {
    ...result,
    reportId: task.reportId,
    reportLabel: task.reportLabel,
    reportOrder: task.reportOrder
  };
}

function recordCommand(state: any, record: any) {
  state.commandRecords.push(record);
  const context = commandContext.getStore();
  context?.commandRecords.push(record);
}

function spawnCommand(executable: any, args: any, options: any) {
  return new Promise((resolve) => {
    let stdout = "";
    let stderr = "";
    let stdoutBytes = 0;
    let stderrBytes = 0;
    let childError: any = null;
    let settled = false;

    const child = spawn(executable, args, {
      cwd: options.cwd,
      env: options.env,
      windowsHide: true,
      stdio: ["pipe", "pipe", "pipe"]
    });

    const appendOutput = (chunk: any, streamName: any) => {
      const text = chunk.toString("utf8");
      const bytes = Buffer.byteLength(text, "utf8");
      if (streamName === "stdout") {
        stdout += text;
        stdoutBytes += bytes;
      } else {
        stderr += text;
        stderrBytes += bytes;
      }
      if (stdoutBytes + stderrBytes > options.maxBuffer && !child.killed) {
        childError = new Error(`command output exceeded ${options.maxBuffer} bytes`);
        child.kill();
      }
    };

    child.stdout.on("data", (chunk) => appendOutput(chunk, "stdout"));
    child.stderr.on("data", (chunk) => appendOutput(chunk, "stderr"));
    child.on("error", (error) => {
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

    function finish(exitCode: any, signal: any) {
      if (settled) {
        return;
      }
      settled = true;
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

function formatTestResult(result: any) {
  const status = result.ok ? "PASS" : "FAIL";
  const error = result.error ? `: ${result.error.message}` : "";
  const duration = Number.isFinite(result.durationMs) ? `, ${result.durationMs}ms` : "";
  return `${status} ${result.label} (${result.commandCount} command(s)${duration})${error}`;
}

function aggregateSmokeReports(results: any) {
  const reports = new Map();

  for (const result of results) {
    const reportId = result.reportId ?? result.id ?? result.label;
    const report = reports.get(reportId) ?? {
      id: reportId,
      label: result.reportLabel ?? result.label,
      reportId,
      reportLabel: result.reportLabel ?? result.label,
      reportOrder: result.reportOrder ?? reports.size,
      ok: true,
      commandCount: 0,
      durationMs: 0,
      startedAtMs: result.startedAtMs,
      endedAtMs: result.endedAtMs
    };

    report.ok &&= result.ok;
    report.commandCount += result.commandCount;
    report.startedAtMs = Math.min(report.startedAtMs, result.startedAtMs);
    report.endedAtMs = Math.max(report.endedAtMs, result.endedAtMs);
    report.durationMs = report.endedAtMs - report.startedAtMs;
    if (!result.ok && !report.error) {
      report.error = result.error;
    }
    reports.set(reportId, report);
  }

  return [...reports.values()].sort((left, right) => left.reportOrder - right.reportOrder);
}

function formatAssertions(assertions: any) {
  if (assertions.length === 0) {
    return ["  (none)"];
  }
  return assertions.map((assertion: any) => `  ${assertion.ok ? "PASS" : "FAIL"} ${assertion.summary}`);
}

function summarizeStdin(stdin: any) {
  if (stdin === undefined || stdin === null) {
    return null;
  }
  const byteCount = Buffer.isBuffer(stdin) ? stdin.length : Buffer.byteLength(String(stdin), "utf8");
  const unit = byteCount === 1 ? "byte" : "bytes";
  return `${byteCount} ${unit} stdin`;
}

function relativeLogPath(root: any, logPath: any) {
  return path.relative(root, logPath).replaceAll(path.sep, "/");
}
