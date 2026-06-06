import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const logDir = path.join(root, ".log", "verify-docnav-workspace");
const timestamp = new Date().toISOString().replace(/[:]/g, "-");
const logPaths = [
  path.join(logDir, "latest.log"),
  path.join(logDir, `${timestamp}.log`)
];

const checks = [
  {
    label: "cargo fmt",
    command: "cargo",
    args: ["fmt", "--all", "--check"]
  },
  {
    label: "generated error rules",
    command: "node",
    args: ["scripts/generate-error-rules.mjs", "--check"],
    ignoreOutput: [
      /^generated error rules ok$/
    ]
  },
  {
    label: "docs validators",
    command: "node",
    args: ["scripts/validate-docs.mjs"],
    ignoreOutput: [
      /\bok(?::|$)/,
      /^schema ok:/,
      /^markdown links ok:/
    ]
  },
  {
    label: "docnav-markdown CLI smoke",
    command: "pnpm",
    args: ["run", "smoke:docnav-markdown"],
    ignoreOutput: [
      /^> docnav-contract-docs@.* smoke:docnav-markdown .*$/,
      /^> node scripts\/with-cargo-bin\.mjs --package docnav-markdown --bin docnav-markdown --env DOCNAV_MARKDOWN_BIN -- node scripts\/docnav-markdown-cli-smoke\.mjs$/,
      /^\$ node scripts\/with-cargo-bin\.mjs --package docnav-markdown --bin docnav-markdown --env DOCNAV_MARKDOWN_BIN -- node scripts\/docnav-markdown-cli-smoke\.mjs$/,
      /^\s*Finished `.*` profile .*$/,
      /^Docnav Markdown CLI Smoke$/,
      /^Status: passed$/,
      /^Commands: \d+$/,
      /^Log:$/,
      /^\s+- \.log\/docnav-markdown-cli-smoke\/latest\.log$/
    ]
  },
  {
    label: "cargo clippy",
    command: "cargo",
    args: ["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
    ignoreOutput: [
      /^\s*Finished `.*` profile .*$/
    ]
  },
  {
    label: "cargo test",
    command: "cargo",
    args: ["test", "--workspace"],
    ignoreOutput: [
      /^\s*Finished `.*` profile .*$/,
      /^\s*Running unittests .*$/,
      /^\s*Running tests[\\/].*$/,
      /^\s*Doc-tests .*$/,
      /^running \d+ tests$/,
      /^test .* \.\.\. ok$/,
      /^test result: ok\..*$/
    ]
  },
  {
    label: "openspec",
    command: "openspec",
    args: ["validate", "--all", "--strict"],
    ignoreOutput: [
      /^✓ /,
      /^Totals: \d+ passed, 0 failed .*$/,
      /^- Validating\.\.\.$/
    ]
  },
  {
    label: "git diff whitespace",
    command: "git",
    args: ["diff", "--check"],
    ignoreOutput: [
      /LF will be replaced by CRLF/i
    ]
  }
];

fs.mkdirSync(logDir, { recursive: true });
for (const logPath of logPaths) {
  fs.writeFileSync(
    logPath,
    [
      "docnav workspace verification",
      `started: ${new Date().toISOString()}`,
      `cwd: ${root}`,
      ""
    ].join("\n"),
    "utf8"
  );
}

const passed = [];
const failed = [];
for (const check of checks) {
  const result = runCheck(check);
  const combined = combineOutput(result);
  appendLog(check, result, combined);

  const visibleLines = lines(combined).filter((line) => !isIgnoredOutput(check, line));
  if (result.error || result.status !== 0) {
    const exitCode = result.status ?? 1;
    failed.push({
      check,
      exitCode,
      error: result.error,
      outputLines: visibleLines
    });
    continue;
  }

  if (visibleLines.length > 0) {
    printOutputSection(check.label, visibleLines);
  }

  passed.push(check.label);
}

for (const logPath of logPaths) {
  fs.appendFileSync(logPath, `completed: ${new Date().toISOString()}\n`, "utf8");
}

if (failed.length > 0) {
  printFailureSummary(passed, failed);
  process.exit(1);
}

printSuccessSummary(passed);

function runCheck(check) {
  const child =
    process.platform === "win32"
      ? {
          command: process.env.ComSpec || "cmd.exe",
          args: ["/d", "/s", "/c", commandLine(check)]
        }
      : check;

  return spawnSync(child.command, child.args, {
    cwd: root,
    encoding: "utf8",
    windowsHide: true,
    maxBuffer: 1024 * 1024 * 64
  });
}

function appendLog(check, result, combined) {
  const section = [
    `## ${check.label}`,
    `$ ${commandLine(check)}`,
    `exit: ${result.status ?? "spawn-error"}`,
    result.error ? `spawn_error: ${result.error.message}` : null,
    "",
    combined || "(no output)",
    "",
    ""
  ]
    .filter((line) => line !== null)
    .join("\n");

  for (const logPath of logPaths) {
    fs.appendFileSync(logPath, section, "utf8");
  }
}

function combineOutput(result) {
  return [result.stdout, result.stderr].filter(Boolean).join("\n");
}

function lines(output) {
  return output.split(/\r?\n/).filter((line) => line.length > 0);
}

function isIgnoredOutput(check, line) {
  return (check.ignoreOutput ?? []).some((pattern) => pattern.test(line));
}

function printSuccessSummary(passedChecks) {
  console.log("");
  console.log("Docnav Workspace Verification");
  console.log("Status: passed");
  console.log("");
  console.log("Checks:");
  for (const check of passedChecks) {
    console.log(`  - ${check}`);
  }
  console.log("");
  console.log("Log:");
  console.log(`  - ${relativeLogPath(logPaths[0])}`);
  console.log("");
}

function printFailureSummary(passedChecks, failedChecks) {
  console.error("");
  console.error("Docnav Workspace Verification");
  console.error("Status: failed");
  console.error("");
  console.error("Failed Checks:");
  for (const failure of failedChecks) {
    console.error(`  - ${failure.check.label} (exit code: ${failure.exitCode})`);
    if (failure.error) {
      console.error(`    ${failure.error.message}`);
    }
  }
  console.error("");
  console.error("Failure Details:");
  for (const failure of failedChecks) {
    console.error(`  - ${failure.check.label}:`);
    if (failure.error) {
      console.error(`    ${failure.error.message}`);
    }
    if (failure.outputLines.length === 0) {
      console.error("    no command output after configured filters");
      continue;
    }
    for (const line of failure.outputLines) {
      console.error(`    ${line}`);
    }
  }
  if (passedChecks.length > 0) {
    console.error("");
    console.error("Passed Checks:");
    for (const check of passedChecks) {
      console.error(`  - ${check}`);
    }
  }
  console.error("");
  printLogLocation();
}

function printLogLocation() {
  console.error("");
  console.error("Log:");
  console.error(`  - ${relativeLogPath(logPaths[0])}`);
  console.error("");
}

function printOutputSection(label, values) {
  console.log("");
  console.log(`Output: ${label}`);
  for (const value of values) {
    console.log(`  ${value}`);
  }
  console.log("");
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

function relativeLogPath(logPath) {
  return path.relative(root, logPath).replaceAll(path.sep, "/");
}
