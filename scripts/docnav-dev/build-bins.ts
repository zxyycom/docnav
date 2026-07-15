import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildCargoExecutables,
  reportCargoExecutableBuildFailure,
  type CargoBinarySpec
} from "../tools/cargo.ts";
import { booleanOption, parseScriptArgs, stringOption } from "../tools/foundation/src/args.ts";
import { writeJsonFile } from "../tools/foundation/src/fs.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const binaries: Array<CargoBinarySpec & { envName: string }> = [
  { packageName: "docnav", binName: "docnav", envName: "DOCNAV_BIN" }
];

export type DevBinarySpec = CargoBinarySpec & { envName: string };

type DevBinOptions = {
  cleanup: boolean;
  copyTo: string | null;
  outputEnvJson: string | null;
  quiet: boolean;
};

if (isMainModule()) {
  const options = parseArgs(process.argv.slice(2));
  cleanupDevBinArtifacts(options);

  if (options.cleanup) {
    console.log("dev binary artifacts cleaned");
  } else {
    const env = buildDevBins(options);

    if (options.outputEnvJson) {
      const envFile = path.resolve(root, options.outputEnvJson);
      writeJsonFile(envFile, env);
    }

    console.log(`dev binaries ok: ${Object.keys(env).join(", ")}`);
  }
}

function parseArgs(args: string[]): DevBinOptions {
  try {
    const parsed = parseScriptArgs({
      args,
      options: {
        cleanup: { type: "boolean" },
        "copy-to": { type: "string" },
        quiet: { type: "boolean" },
        "output-env-json": { type: "string" }
      }
    });

    return {
      cleanup: booleanOption(parsed.values, "cleanup"),
      copyTo: stringOption(parsed.values, "copy-to") ?? null,
      outputEnvJson: stringOption(parsed.values, "output-env-json") ?? null,
      quiet: booleanOption(parsed.values, "quiet")
    };
  } catch (error: unknown) {
    usage(error instanceof Error ? error.message : String(error));
  }
}

function buildDevBins(options: DevBinOptions): Record<string, string> {
  const result = buildCargoExecutables({ binaries, cwd: root });

  if (!result.ok) {
    process.exit(reportCargoExecutableBuildFailure(result));
  }

  if (result.stderr && !options.quiet) {
    process.stderr.write(result.stderr);
  }

  return prepareDevBinEnv({
    binaries,
    copyTo: options.copyTo ? path.resolve(root, options.copyTo) : null,
    executables: result.executables
  });
}

export function prepareDevBinEnv({
  binaries,
  copyTo,
  executables
}: {
  binaries: readonly DevBinarySpec[];
  copyTo?: string | null;
  executables: ReadonlyMap<string, string>;
}): Record<string, string> {
  const resolvedExecutables = copyTo ? copyDevBinExecutables(binaries, executables, copyTo) : executables;
  return Object.fromEntries(
    binaries.map((binary) => [binary.envName, executablePathFor(resolvedExecutables, binary)])
  );
}

export function cleanupDevBinArtifacts({
  copyTo,
  outputEnvJson
}: Pick<DevBinOptions, "copyTo" | "outputEnvJson">): void {
  if (outputEnvJson) {
    fs.rmSync(path.resolve(root, outputEnvJson), { force: true });
  }
  if (copyTo) {
    fs.rmSync(path.resolve(root, copyTo), { force: true, recursive: true });
  }
}

function copyDevBinExecutables(
  binaries: readonly DevBinarySpec[],
  executables: ReadonlyMap<string, string>,
  copyRoot: string
): Map<string, string> {
  fs.mkdirSync(copyRoot, { recursive: true });
  const runDir = fs.mkdtempSync(path.join(copyRoot, "run-"));
  const copied = new Map<string, string>();

  for (const binary of binaries) {
    const sourcePath = executablePathFor(executables, binary);
    const destPath = path.join(runDir, path.basename(sourcePath));
    fs.copyFileSync(sourcePath, destPath);
    fs.chmodSync(destPath, fs.statSync(sourcePath).mode);
    copied.set(binary.binName, destPath);
  }

  return copied;
}

function executablePathFor(executables: ReadonlyMap<string, string>, binary: DevBinarySpec): string {
  const executable = executables.get(binary.binName);
  if (!executable) {
    throw new Error(`missing built executable for ${binary.binName}`);
  }
  return executable;
}

function usage(message: string): never {
  console.error(message);
  console.error("usage: bun scripts/docnav-dev/build-bins.ts [--cleanup] [--quiet] [--output-env-json <path>] [--copy-to <dir>]");
  process.exit(2);
}

function isMainModule() {
  return process.argv[1] ? path.resolve(process.argv[1]) === fileURLToPath(import.meta.url) : false;
}
