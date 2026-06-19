import path from "node:path";
import { fileURLToPath } from "node:url";

import { findCargoExecutable } from "./tools/cargo.ts";
import { booleanOption, parseScriptArgs, stringOption } from "./tools/args.ts";
import { processFailed, runProcessSync, writeProcessOutput } from "./tools/process.ts";
import { writeJsonFile } from "./tools/fs.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const binaries = [
  { packageName: "docnav", binName: "docnav", envName: "DOCNAV_BIN" },
  { packageName: "docnav-markdown", binName: "docnav-markdown", envName: "DOCNAV_MARKDOWN_BIN" }
];

type DevBinOptions = {
  outputEnvJson: string | null;
  quiet: boolean;
};

const options = parseArgs(process.argv.slice(2));
const env = buildDevBins(options.quiet);

if (options.outputEnvJson) {
  const envFile = path.resolve(root, options.outputEnvJson);
  writeJsonFile(envFile, env);
}

console.log(`dev binaries ok: ${Object.keys(env).join(", ")}`);

function parseArgs(args: string[]): DevBinOptions {
  try {
    const parsed = parseScriptArgs({
      args,
      options: {
        quiet: { type: "boolean" },
        "output-env-json": { type: "string" }
      }
    });

    return {
      outputEnvJson: stringOption(parsed.values, "output-env-json") ?? null,
      quiet: booleanOption(parsed.values, "quiet")
    };
  } catch (error: unknown) {
    usage(error instanceof Error ? error.message : String(error));
  }
}

function buildDevBins(quiet: boolean): Record<string, string> {
  const cargoArgs = [
    "build",
    ...binaries.flatMap((binary) => ["-p", binary.packageName]),
    ...binaries.flatMap((binary) => ["--bin", binary.binName]),
    "--message-format=json"
  ];
  const result = runProcessSync("cargo", cargoArgs, {
    cwd: root,
    maxBuffer: 1024 * 1024 * 64
  });

  if (processFailed(result)) {
    writeProcessOutput(result);
    if (result.error) {
      console.error(result.error.message);
    }
    process.exit(result.status ?? 1);
  }

  if (result.stderr && !quiet) {
    process.stderr.write(result.stderr);
  }

  const env: Record<string, string> = {};
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

function usage(message: string): never {
  console.error(message);
  console.error("usage: node scripts/build-docnav-dev-bins.ts [--quiet] [--output-env-json <path>]");
  process.exit(2);
}
