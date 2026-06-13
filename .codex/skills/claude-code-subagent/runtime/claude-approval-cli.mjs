#!/usr/bin/env node

import {
  decideRequest,
  getStatus,
  startSession,
  stopSession,
} from "./approval-session-controller.mjs";
import {
  formatCommandOutput,
  formatError,
} from "./approval-session-output.mjs";

function printJson(value) {
  process.stdout.write(`${JSON.stringify(value, null, 2)}\n`);
}

function printJsonError(value) {
  process.stderr.write(`${JSON.stringify(value, null, 2)}\n`);
}

function printText(value) {
  process.stdout.write(value);
}

function fail(message) {
  throw new Error(message);
}

const MAX_WAIT_SECONDS = 1800;

function parseArguments(argv) {
  const options = {};
  const positionals = [];

  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (!argument.startsWith("--")) {
      positionals.push(argument);
      continue;
    }

    const separator = argument.indexOf("=");
    if (separator !== -1) {
      options[argument.slice(2, separator)] = argument.slice(separator + 1);
      continue;
    }

    const name = argument.slice(2);
    const value = argv[index + 1];
    if (value === undefined || value.startsWith("--")) {
      options[name] = true;
      continue;
    }
    options[name] = value;
    index += 1;
  }

  return { command: positionals[0], options };
}

function requireString(options, name) {
  const value = options[name];
  if (typeof value !== "string" || value.length === 0) {
    fail(`Provide --${name}.`);
  }
  return value;
}

function parseWaitSeconds(value) {
  if (value === undefined) return 0;
  const waitSeconds = Number(value);
  if (
    !Number.isInteger(waitSeconds) ||
    waitSeconds < 0 ||
    waitSeconds > MAX_WAIT_SECONDS
  ) {
    fail(`--wait-seconds must be an integer from 0 through ${MAX_WAIT_SECONDS}.`);
  }
  return waitSeconds;
}

function parseUuid(value, name) {
  if (typeof value !== "string" || value.length === 0) {
    fail(`Provide --${name}.`);
  }
  if (
    !/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/iu.test(
      value,
    )
  ) {
    fail(`--${name} must be a UUID.`);
  }
  return value;
}

function optionalUuid(options, name) {
  const value = options[name];
  if (value === undefined) return undefined;
  return parseUuid(value, name);
}

function requireUuid(options, name) {
  return parseUuid(options[name], name);
}

function rejectSessionDirectory(options) {
  if (options["session-directory"] !== undefined) {
    fail("Use --session-id instead of --session-directory.");
  }
}

function parseUpdatedInput(value) {
  if (value === undefined) return undefined;
  let updatedInput;
  try {
    updatedInput = JSON.parse(value);
  } catch {
    fail("--updated-input-json must be valid JSON.");
  }
  if (
    updatedInput === null ||
    typeof updatedInput !== "object" ||
    Array.isArray(updatedInput)
  ) {
    fail("--updated-input-json must be a JSON object.");
  }
  return updatedInput;
}

function printHelp() {
  process.stdout.write(`Usage:
  node claude-approval-cli.mjs start --working-directory <path> (--prompt <text> | --prompt-file <path>) [--permission-mode auto|acceptEdits|default|plan] [--claude-executable <path>] [--json]
  node claude-approval-cli.mjs status [--session-id <uuid>] [--wait-seconds 0..${MAX_WAIT_SECONDS}] [--json]
  node claude-approval-cli.mjs approve --request-id <uuid> [--reason <text>] [--updated-input-json <json>] [--json]
  node claude-approval-cli.mjs deny --request-id <uuid> [--reason <text>] [--message <text>] [--json]
  node claude-approval-cli.mjs stop [--session-id <uuid>] [--reason <text>] [--json]
`);
}

async function runCommand(command, options) {
  switch (command) {
    case "start":
      return startSession({
        workingDirectory: requireString(options, "working-directory"),
        prompt: options.prompt,
        promptFile: options["prompt-file"],
        permissionMode: options["permission-mode"],
        claudeExecutable: options["claude-executable"],
      });
    case "status":
      rejectSessionDirectory(options);
      return getStatus({
        sessionId: optionalUuid(options, "session-id"),
        waitSeconds: parseWaitSeconds(options["wait-seconds"]),
      });
    case "approve":
      rejectSessionDirectory(options);
      return decideRequest({
        requestId: requireUuid(options, "request-id"),
        behavior: "allow",
        reason: options.reason,
        updatedInput: parseUpdatedInput(options["updated-input-json"]),
      });
    case "deny":
      rejectSessionDirectory(options);
      return decideRequest({
        requestId: requireUuid(options, "request-id"),
        behavior: "deny",
        reason: options.reason,
        message: options.message,
      });
    case "stop":
      rejectSessionDirectory(options);
      return stopSession({
        sessionId: optionalUuid(options, "session-id"),
        reason: options.reason,
      });
    default:
      fail(`Unknown command: ${command}`);
  }
}

async function main() {
  let jsonOutput = false;
  try {
    const { command, options } = parseArguments(process.argv.slice(2));
    if (!command || command === "help" || options.help === true) {
      printHelp();
      return;
    }
    if (options.json !== undefined && options.json !== true) {
      fail("--json does not take a value.");
    }
    jsonOutput = options.json === true;

    const result = await runCommand(command, options);
    if (jsonOutput) {
      printJson(result);
    } else {
      printText(formatCommandOutput(command, result));
    }
  } catch (error) {
    if (jsonOutput) {
      printJsonError({
        error: {
          name: error?.name || "Error",
          message: error?.message || String(error),
        },
      });
    } else {
      process.stderr.write(formatError(error));
    }
    process.exitCode = 1;
  }
}

void main();
