import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildCargoExecutables,
  reportCargoExecutableBuildFailure,
  type CargoBinarySpec
} from "../tools/cargo.ts";
import { booleanOption, parseScriptArgs, stringOption } from "../tools/args.ts";
import { writeJsonFile } from "../tools/fs.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const binaries: Array<CargoBinarySpec & { envName: string }> = [
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
  const result = buildCargoExecutables({ binaries, cwd: root });

  if (!result.ok) {
    process.exit(reportCargoExecutableBuildFailure(result));
  }

  if (result.stderr && !quiet) {
    process.stderr.write(result.stderr);
  }

  return Object.fromEntries(
    binaries.map((binary) => [binary.envName, result.executables.get(binary.binName)!])
  );
}

function usage(message: string): never {
  console.error(message);
  console.error("usage: bun scripts/docnav-dev/build-bins.ts [--quiet] [--output-env-json <path>]");
  process.exit(2);
}
