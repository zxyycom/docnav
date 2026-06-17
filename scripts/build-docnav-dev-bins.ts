import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { findCargoExecutable } from "./tools/cargo.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const binaries = [
  { packageName: "docnav", binName: "docnav", envName: "DOCNAV_BIN" },
  { packageName: "docnav-markdown", binName: "docnav-markdown", envName: "DOCNAV_MARKDOWN_BIN" }
];

const options = parseArgs(process.argv.slice(2));
const env = buildDevBins(options.quiet);

if (options.outputEnvJson) {
  const envFile = path.resolve(root, options.outputEnvJson);
  fs.mkdirSync(path.dirname(envFile), { recursive: true });
  fs.writeFileSync(envFile, `${JSON.stringify(env, null, 2)}\n`, "utf8");
}

console.log(`dev binaries ok: ${Object.keys(env).join(", ")}`);

function parseArgs(args: any) {
  const options = {
    outputEnvJson: null,
    quiet: false
  };

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--quiet") {
      options.quiet = true;
      continue;
    }
    if (arg === "--output-env-json") {
      const value = args[index + 1];
      if (!value) {
        usage("--output-env-json requires a value");
      }
      options.outputEnvJson = value;
      index += 1;
      continue;
    }
    if (arg.startsWith("--output-env-json=")) {
      options.outputEnvJson = arg.slice("--output-env-json=".length);
      continue;
    }
    usage(`unknown option ${arg}`);
  }

  return options;
}

function buildDevBins(quiet: any) {
  const cargoArgs = [
    "build",
    ...binaries.flatMap((binary) => ["-p", binary.packageName]),
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

  const env: Record<string, any> = {};
  for (const binary of binaries) {
    const executable = findCargoExecutable(result.stdout ?? "", binary.binName);
    if (!executable) {
      console.error(`cargo build did not report a ${binary.binName} executable`);
      process.exit(1);
    }
    env[binary.envName] = executable;
  }
  return env;
}

function writeOutput(result: any) {
  if (result.stdout) {
    process.stdout.write(result.stdout);
  }
  if (result.stderr) {
    process.stderr.write(result.stderr);
  }
}

function usage(message: any) {
  console.error(message);
  console.error("usage: node scripts/build-docnav-dev-bins.ts [--quiet] [--output-env-json <path>]");
  process.exit(2);
}
