import { execFile } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

import { expandTasks, runParallelTasks } from "./tools/parallel-task-runner.mjs";

const execFileAsync = promisify(execFile);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const logDir = path.join(root, ".log", "verify-docnav-workspace");
const MAX_BUFFER = 1024 * 1024 * 64;
const DEV_BIN_ENV_FILE = ".log/verify-docnav-workspace/dev-bins.json";

export const PROFILE_REQUIRED = "required";
export const PROFILE_FULL = "full";

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
        id: "generated-error-rules",
        label: "generated error rules",
        command: "node",
        args: ["scripts/generate-error-rules.mjs", "--check"],
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
          ["workspace-verifier-tests", "workspace verifier tests", "scripts/tools/verify-docnav-workspace.test.mjs"],
          ["smoke-harness-tests", "smoke harness tests", "test/tools/smoke-harness.test.mjs"],
          ["parallel-task-runner-tests", "parallel task runner tests", "scripts/tools/parallel-task-runner.test.mjs"]
        ])
      },
      {
        id: "release-package-script-tests",
        label: "release package script tests",
        command: "node",
        args: ["--test", "scripts/tools/release-package/args.test.mjs"],
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
          ["quality-tools-tests", "quality tools tests", "scripts/tools/quality/tools.test.mjs"]
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
            args: ["scripts/build-docnav-dev-bins.mjs", "--quiet", "--output-env-json", DEV_BIN_ENV_FILE],
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
                args: ["test/docnav-markdown-smoke.mjs"],
                ignoreOutput: [
                  ...smokeSuccessOutput("Docnav Markdown Development Smoke", ".log/docnav-markdown-cli-smoke/latest.log")
                ]
              },
              {
                id: "docnav-core-development-smoke",
                label: "docnav core development smoke",
                command: "node",
                args: ["test/docnav-core-smoke.mjs"],
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

export function parseArgs(argv) {
  const options = {
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
      options.profile = value;
      index += 1;
      continue;
    }
    if (arg.startsWith("--profile=")) {
      options.profile = arg.slice("--profile=".length);
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

  assertProfile(options.profile);
  options.concurrency = resolveVerificationConcurrency(options.concurrency);
  return options;
}

export function checksForProfile(profile) {
  assertProfile(profile);
  if (profile === PROFILE_REQUIRED) {
    return checks.filter((check) => check.type === PROFILE_REQUIRED);
  }
  return checks.filter((check) => check.type === PROFILE_REQUIRED || check.type === PROFILE_FULL);
}

export function visibleOutputLines(check, output) {
  return lines(output).filter((line) => !isIgnoredOutput(check, line));
}

export function isIgnoredOutput(check, line) {
  return (check.ignoreOutput ?? []).some((pattern) => pattern.test(line));
}

export function formatCompletionLine(result) {
  return `  ${result.ok ? "passed" : "failed"}: ${result.check.label} (${formatDurationMs(result.durationMs)})`;
}

export function reportCountForChecks(checkList) {
  return new Set(checkList.map(reportIdForCheck)).size;
}

export function formatDurationMs(durationMs) {
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
  } catch (error) {
    console.error(error.message);
    printUsage(console.error);
    process.exitCode = 2;
  }
}

async function runVerification({ profile, concurrency }) {
  const selectedChecks = checksForProfile(profile);
  const totalReports = reportCountForChecks(selectedChecks);
  const completeReport = createReportCompletionTracker(selectedChecks);
  const logPaths = createLogPaths();
  initializeLogs(logPaths, profile, totalReports, selectedChecks.length);

  const startedAtMs = Date.now();
  const completedResults = [];
  const completedReports = [];

  printHeader(profile, totalReports);
  await runParallelTasks(selectedChecks, {
    concurrency,
    execute: executeCheck,
    onComplete: (result) => {
      completedResults.push(result);
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

async function executeCheck(check) {
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
  } catch (error) {
    return buildCheckResult(check, {
      ok: false,
      exitCode: normalizeExitCode(error),
      stdout: typeof error.stdout === "string" ? error.stdout : "",
      stderr: typeof error.stderr === "string" ? error.stderr : "",
      error,
      startedAtMs,
      endedAtMs: Date.now()
    });
  }
}

function buildCheckResult(check, data) {
  const combinedOutput = [data.stdout, data.stderr].filter(Boolean).join("\n");
  return {
    check,
    ok: data.ok,
    exitCode: data.exitCode,
    error: data.error ?? null,
    stdout: data.stdout,
    stderr: data.stderr,
    combinedOutput,
    durationMs: data.endedAtMs - data.startedAtMs,
    startedAtMs: data.startedAtMs,
    endedAtMs: data.endedAtMs
  };
}

function normalizeExitCode(error) {
  return typeof error?.code === "number" ? error.code : 1;
}

function commandInvocation(check) {
  if (process.platform !== "win32") {
    return check;
  }

  return {
    command: process.env.ComSpec || "cmd.exe",
    args: ["/d", "/s", "/c", commandLine(check)]
  };
}

function environmentForCheck(check) {
  return {
    ...process.env,
    ...readEnvFile(check.envFile),
    ...(check.env ?? {})
  };
}

function readEnvFile(envFile) {
  if (!envFile) {
    return {};
  }
  const envPath = path.resolve(root, envFile);
  const parsed = JSON.parse(fs.readFileSync(envPath, "utf8"));
  return Object.fromEntries(
    Object.entries(parsed).map(([key, value]) => [key, String(value)])
  );
}

function printHeader(profile, totalChecks) {
  console.log("");
  console.log("Docnav Workspace Verification");
  console.log(`Profile: ${profile}`);
  console.log(`Total checks: ${totalChecks}`);
  console.log("");
  console.log("Checks:");
}

function printSummary({ profile, totalChecks, completedResults, totalDurationMs, logPaths }) {
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

function appendLog(logPaths, result) {
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

function createLogPaths() {
  const timestamp = new Date().toISOString().replace(/[:]/g, "-");
  return [
    path.join(logDir, "latest.log"),
    path.join(logDir, `${timestamp}.log`)
  ];
}

function initializeLogs(logPaths, profile, totalChecks, leafChecks) {
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

function finalizeLogs(logPaths, totalDurationMs) {
  for (const logPath of logPaths) {
    fs.appendFileSync(logPath, `completed: ${new Date().toISOString()}\n`, "utf8");
    fs.appendFileSync(logPath, `duration: ${formatDurationMs(totalDurationMs)}\n`, "utf8");
  }
}

function defineChecks(checkList) {
  return withCheckReportMetadata(checkList).map((check) => ({
    args: [],
    ignoreOutput: [],
    ...check
  }));
}

function withCheckReportMetadata(checkList) {
  return expandTasks(checkList.map((check) => annotateCheckReport(check, null)));
}

function annotateCheckReport(check, inheritedReport) {
  const report = inheritedReport ?? (typeof check.label === "string" ? createCheckReport(check) : null);
  if (Array.isArray(check.tasks)) {
    return {
      ...check,
      tasks: check.tasks.map((child) => annotateCheckReport(child, report))
    };
  }
  const leafReport = report ?? createCheckReport(check);
  return {
    ...check,
    reportId: leafReport.id,
    reportLabel: leafReport.label
  };
}

function createCheckReport(check) {
  return {
    id: check.id,
    label: check.label ?? check.id
  };
}

function createReportCompletionTracker(checkList) {
  const reports = new Map();
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

  return (result) => {
    const report = reports.get(reportIdForCheck(result.check));
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

function reportIdForCheck(check) {
  return check.reportId ?? check.id;
}

function reportLabelForCheck(check) {
  return check.reportLabel ?? check.label;
}

function docsValidatorChecks() {
  return [
    {
      id: "docs-json-validator",
      label: "docs json validator",
      command: "node",
      args: ["scripts/validate-docs.mjs", "json"],
      ignoreOutput: [
        /^json syntax ok:/
      ]
    },
    {
      id: "docs-schema-validator",
      label: "docs schema validator",
      command: "node",
      args: ["scripts/validate-docs.mjs", "schema"],
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
      args: ["scripts/validate-docs.mjs", "mcp"],
      ignoreOutput: [
        /^mcp structuredContent ok:/
      ]
    },
    {
      id: "docs-semantics-validator",
      label: "docs semantics validator",
      command: "node",
      args: ["scripts/validate-docs.mjs", "semantics"],
      ignoreOutput: [
        /^protocol\/readable mapping ok:/,
        /^error details ok:/,
        /^manifest semantics ok:/,
        /^MCP bridge handoff docs ok:/,
        /^document output mode consistency ok:/
      ]
    },
    {
      id: "docs-links-validator",
      label: "docs links validator",
      command: "node",
      args: ["scripts/validate-docs.mjs", "links"],
      ignoreOutput: [
        /^markdown links ok:/
      ]
    }
  ];
}

function nodeTestFileChecks(testFiles) {
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

function smokeSuccessOutput(title, logPath) {
  return [
    new RegExp(`^${escapeRegex(title)}$`),
    /^Status: passed$/,
    /^Commands: \d+$/,
    /^Log:$/,
    new RegExp(`^\\s+- ${escapeRegex(logPath)}$`)
  ];
}

function commandLine(check) {
  return [check.command, ...check.args].map(quoteArg).join(" ");
}

function quoteArg(value) {
  if (/^[A-Za-z0-9_./:=@+-]+$/.test(value)) {
    return value;
  }
  return JSON.stringify(value);
}

function lines(output) {
  return output.split(/\r?\n/).filter((line) => line.length > 0);
}

function escapeRegex(value) {
  return String(value).replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function assertProfile(profile) {
  if (!Object.hasOwn(profiles, profile)) {
    throw new Error(`unknown verification profile: ${profile}`);
  }
}

function printUsage(writeLine) {
  writeLine("Usage: node scripts/verify-docnav-workspace.mjs [--profile required|full] [--concurrency <n>]");
  writeLine("");
  writeLine("Profiles:");
  for (const [name, profile] of Object.entries(profiles)) {
    writeLine(`  - ${name}: ${profile.description}`);
  }
}

function relativeLogPath(logPath) {
  return path.relative(root, logPath).replaceAll(path.sep, "/");
}

function isMainModule() {
  return process.argv[1] ? path.resolve(process.argv[1]) === fileURLToPath(import.meta.url) : false;
}
