import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { findCargoExecutable } from "./cargo.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const options = parseArgs(process.argv.slice(2));
const executable = buildCargoBin(options.packageName, options.binName);
const env = {
  ...process.env,
  [options.envName]: executable
};

const result = spawnSync(options.command[0], options.command.slice(1), {
  cwd: root,
  env,
  stdio: "inherit",
  windowsHide: true
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);

function parseArgs(args) {
  const separatorIndex = args.indexOf("--");
  if (separatorIndex === -1) {
    usage("missing -- command separator");
  }

  const optionArgs = args.slice(0, separatorIndex);
  const command = args.slice(separatorIndex + 1);
  if (command.length === 0) {
    usage("missing command after --");
  }

  const parsed = {
    packageName: null,
    binName: null,
    envName: "CARGO_BIN_EXE",
    command
  };

  for (let index = 0; index < optionArgs.length; index += 1) {
    const flag = optionArgs[index];
    const value = optionArgs[index + 1];
    if (!value) {
      usage(`${flag} requires a value`);
    }

    switch (flag) {
      case "--package":
        parsed.packageName = value;
        break;
      case "--bin":
        parsed.binName = value;
        break;
      case "--env":
        parsed.envName = value;
        break;
      default:
        usage(`unknown option ${flag}`);
    }
    index += 1;
  }

  if (!parsed.packageName) {
    usage("--package is required");
  }
  if (!parsed.binName) {
    usage("--bin is required");
  }
  if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(parsed.envName)) {
    usage("--env must be a valid environment variable name");
  }

  return parsed;
}

function buildCargoBin(packageName, binName) {
  const result = spawnSync(
    "cargo",
    ["build", "-p", packageName, "--bin", binName, "--message-format=json"],
    {
      cwd: root,
      encoding: "utf8",
      windowsHide: true,
      maxBuffer: 1024 * 1024 * 64
    }
  );

  if (result.error || result.status !== 0) {
    writeOutput(result);
    if (result.error) {
      console.error(result.error.message);
    }
    process.exit(result.status ?? 1);
  }

  if (result.stderr) {
    process.stderr.write(result.stderr);
  }

  const executable = findCargoExecutable(result.stdout ?? "", binName);
  if (!executable) {
    console.error(`cargo build did not report a ${binName} executable`);
    process.exit(1);
  }

  return executable;
}

function writeOutput(result) {
  if (result.stdout) {
    process.stdout.write(result.stdout);
  }
  if (result.stderr) {
    process.stderr.write(result.stderr);
  }
}

function usage(message) {
  console.error(message);
  console.error(
    "usage: node scripts/with-cargo-bin.mjs --package <cargo-package> --bin <bin-name> --env <ENV_NAME> -- <command> [args...]"
  );
  process.exit(2);
}
