import { Buffer } from "node:buffer";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { compileRegisteredSchema, createSchemaAjv, formatAjvErrors } from "./validators/schema-registry.mjs";

const MAX_COMMAND_OUTPUT = 1024 * 1024 * 64;

export function createSmokeState(values = {}) {
  return {
    commandRecords: [],
    testResults: [],
    startedAt: new Date(),
    validators: null,
    ...values
  };
}

export function argValue(flag) {
  const index = process.argv.indexOf(flag);
  return index === -1 ? null : (process.argv[index + 1] ?? null);
}

export function resolveBinaryPath(root, value) {
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

export function createSmokeHarness(options) {
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
    safeArgPattern = /^[A-Za-z0-9_./:=@+\-]+$/
  } = options;

  function runTest(label, fn) {
    const startedCommands = state.commandRecords.length;
    try {
      fn();
      state.testResults.push({
        label,
        ok: true,
        commandCount: state.commandRecords.length - startedCommands
      });
    } catch (error) {
      state.testResults.push({
        label,
        ok: false,
        commandCount: state.commandRecords.length - startedCommands,
        error
      });
      throw error;
    }
  }

  function runCli(name, args, commandOptions = {}) {
    const cwd = resolveCwd(commandOptions);
    const result = spawnSync(binaryPath(), args, {
      cwd,
      env: resolveEnv(commandOptions),
      input: commandOptions.stdin,
      encoding: "utf8",
      windowsHide: true,
      maxBuffer: MAX_COMMAND_OUTPUT
    });
    const record = {
      name,
      args,
      cwd,
      stdinSummary: commandOptions.stdinSummary ?? summarizeStdin(commandOptions.stdin),
      exitCode: result.status ?? 1,
      signal: result.signal,
      error: result.error?.message ?? null,
      stdout: result.stdout ?? "",
      stderr: result.stderr ?? "",
      assertions: []
    };
    state.commandRecords.push(record);
    if (record.error) {
      expect(record, false, `process spawned successfully: ${record.error}`);
    }
    return record;
  }

  function compileSchemas() {
    const ajv = createSchemaAjv();
    return Object.fromEntries(
      Object.entries(schemaPaths).map(([name, relativePath]) => [name, compileRegisteredSchema(ajv, relativePath)])
    );
  }

  function validateSchema(record, name, value) {
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

  function formatCommandRecord(record) {
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

  function printFailureSummary(error) {
    console.error("");
    console.error(title);
    console.error("Status: failed");
    console.error(`Failure: ${error.message}`);
    printLogLocation(console.error);
  }

  function quoteArg(value) {
    return safeArgPattern.test(value) ? value : JSON.stringify(value);
  }

  function printLogLocation(writeLine) {
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
    runTest,
    validateSchema,
    writeAuditLogs
  };
}

function formatTestResult(result) {
  const status = result.ok ? "PASS" : "FAIL";
  const error = result.error ? `: ${result.error.message}` : "";
  return `${status} ${result.label} (${result.commandCount} command(s))${error}`;
}

function formatAssertions(assertions) {
  if (assertions.length === 0) {
    return ["  (none)"];
  }
  return assertions.map((assertion) => `  ${assertion.ok ? "PASS" : "FAIL"} ${assertion.summary}`);
}

function summarizeStdin(stdin) {
  if (stdin === undefined || stdin === null) {
    return null;
  }
  const byteCount = Buffer.isBuffer(stdin) ? stdin.length : Buffer.byteLength(String(stdin), "utf8");
  const unit = byteCount === 1 ? "byte" : "bytes";
  return `${byteCount} ${unit} stdin`;
}

function relativeLogPath(root, logPath) {
  return path.relative(root, logPath).replaceAll(path.sep, "/");
}
