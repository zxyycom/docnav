import { spawnSync } from "node:child_process";
import type { SpawnSyncReturns } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { findCargoExecutable } from "./tools/cargo.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

type BinarySpec = {
  packageName: string;
  binName: string;
  envName: string;
};

type Options = {
  binaries: BinarySpec[];
  command: [string, ...string[]];
  quiet: boolean;
};

const options = parseArgs(process.argv.slice(2));
const executables = buildCargoBins(options.binaries, options.quiet);
const env = { ...process.env };
for (const binary of options.binaries) {
  const executable = executables.get(binary.binName);
  if (!executable) {
    console.error(`cargo build did not report a ${binary.binName} executable`);
    process.exit(1);
  }
  env[binary.envName] = executable;
}

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

function parseArgs(args: string[]): Options {
  const separatorIndex = args.indexOf("--");
  if (separatorIndex === -1) {
    usage("missing -- command separator");
  }

  const optionArgs = args.slice(0, separatorIndex);
  const command = args.slice(separatorIndex + 1);
  if (command.length === 0) {
    usage("missing command after --");
  }

  const binaries: BinarySpec[] = [];
  let quiet = false;
  for (let index = 0; index < optionArgs.length;) {
    const flag = optionArgs[index];
    if (flag === "--quiet") {
      quiet = true;
      index += 1;
      continue;
    }

    const packageName = optionArgs[index + 1];
    const binName = optionArgs[index + 2];
    const envName = optionArgs[index + 3];
    if (flag !== "--bin") {
      usage(`unknown option ${String(flag)}`);
    }
    if (!packageName || !binName || !envName) {
      usage("--bin requires <cargo-package> <bin-name> <ENV_NAME>");
    }
    if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(envName)) {
      usage(`${envName} must be a valid environment variable name`);
    }
    binaries.push({ packageName, binName, envName });
    index += 4;
  }

  if (binaries.length === 0) {
    usage("at least one --bin declaration is required");
  }
  if (new Set(binaries.map((binary) => binary.binName)).size !== binaries.length) {
    usage("bin names must be unique");
  }
  if (new Set(binaries.map((binary) => binary.envName)).size !== binaries.length) {
    usage("environment variable names must be unique");
  }

  return { binaries, command: command as [string, ...string[]], quiet };
}

function buildCargoBins(binaries: BinarySpec[], quiet: boolean): Map<string, string> {
  const packages = [...new Set(binaries.map((binary) => binary.packageName))];
  const cargoArgs = [
    "build",
    ...packages.flatMap((packageName) => ["-p", packageName]),
    ...binaries.flatMap((binary) => ["--bin", binary.binName]),
    "--message-format=json"
  ];
  const result = spawnSync("cargo", cargoArgs, {
    cwd: root,
    encoding: "utf8",
    windowsHide: true,
    maxBuffer: 1024 * 1024 * 64
  });

  if (result.error || result.status !== 0) {
    writeOutput(result);
    if (result.error) {
      console.error(result.error.message);
    }
    process.exit(result.status ?? 1);
  }

  if (result.stderr && !quiet) {
    process.stderr.write(result.stderr);
  }

  const executables = new Map<string, string>();
  for (const binary of binaries) {
    const executable = findCargoExecutable(result.stdout ?? "", binary.binName);
    if (!executable) {
      console.error(`cargo build did not report a ${binary.binName} executable`);
      process.exit(1);
    }
    executables.set(binary.binName, executable);
  }
  return executables;
}

function writeOutput(result: SpawnSyncReturns<string>) {
  if (result.stdout) {
    process.stdout.write(result.stdout);
  }
  if (result.stderr) {
    process.stderr.write(result.stderr);
  }
}

function usage(message: string): never {
  console.error(message);
  console.error(
    "usage: node scripts/with-cargo-bins.ts [--quiet] --bin <cargo-package> <bin-name> <ENV_NAME> [--bin ...] -- <command> [args...]"
  );
  process.exit(2);
}
