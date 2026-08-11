import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildCargoExecutables,
  reportCargoExecutableBuildFailure
} from "../tools/cargo.ts";
import { booleanOption, parseScriptArgs } from "../tools/foundation/src/args.ts";
import { writeJsonFile } from "../tools/foundation/src/fs.ts";
import {
  assertManagedDevBinArtifactPaths,
  resolveDevBinArtifactPaths
} from "./artifacts.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const docnavBinary = { packageName: "docnav", binName: "docnav" };

type DevBinOptions = {
  cleanup: boolean;
  quiet: boolean;
};

if (isMainModule()) {
  const options = parseArgs(process.argv.slice(2));
  cleanupDevBinArtifacts();

  if (options.cleanup) {
    console.log("dev binary artifacts cleaned");
  } else {
    const env = buildDevBins(options);
    const { envFile } = resolveDevBinArtifactPaths(root);
    writeJsonFile(envFile, env);

    console.log(`dev binaries ok: ${Object.keys(env).join(", ")}`);
  }
}

function parseArgs(args: string[]): DevBinOptions {
  try {
    const parsed = parseScriptArgs({
      args,
      options: {
        cleanup: { type: "boolean" },
        quiet: { type: "boolean" }
      }
    });

    return {
      cleanup: booleanOption(parsed.values, "cleanup"),
      quiet: booleanOption(parsed.values, "quiet")
    };
  } catch (error: unknown) {
    usage(error instanceof Error ? error.message : String(error));
  }
}

function buildDevBins(options: DevBinOptions): Record<string, string> {
  const result = buildCargoExecutables({ binaries: [docnavBinary], cwd: root });

  if (!result.ok) {
    process.exit(reportCargoExecutableBuildFailure(result));
  }

  if (result.stderr && !options.quiet) {
    process.stderr.write(result.stderr);
  }

  const executable = result.executables.get(docnavBinary.binName);
  if (!executable) {
    console.error("cargo build did not report a docnav executable");
    process.exit(1);
  }

  return prepareDevBinEnv({
    docnavExecutable: executable,
    workspaceRoot: root
  });
}

export function prepareDevBinEnv({
  docnavExecutable,
  workspaceRoot = root
}: {
  docnavExecutable: string;
  workspaceRoot?: string;
}): { DOCNAV_BIN: string } {
  const { copyRoot } = resolveDevBinArtifactPaths(workspaceRoot);
  return {
    DOCNAV_BIN: copyDevBinExecutable(docnavExecutable, copyRoot)
  };
}

export function cleanupDevBinArtifacts(workspaceRoot = root): void {
  const paths = resolveDevBinArtifactPaths(workspaceRoot);
  assertManagedDevBinArtifactPaths(paths);
  fs.rmSync(paths.envFile, { force: true });
  fs.rmSync(paths.copyRoot, { force: true, recursive: true });
}

function copyDevBinExecutable(
  sourcePath: string,
  copyRoot: string
): string {
  fs.mkdirSync(copyRoot, { recursive: true });
  const runDir = fs.mkdtempSync(path.join(copyRoot, "run-"));
  const destPath = path.join(runDir, path.basename(sourcePath));
  fs.copyFileSync(sourcePath, destPath);
  fs.chmodSync(destPath, fs.statSync(sourcePath).mode);
  return destPath;
}

function usage(message: string): never {
  console.error(message);
  console.error("usage: bun scripts/docnav-dev/build-bins.ts [--cleanup] [--quiet]");
  process.exit(2);
}

function isMainModule() {
  return process.argv[1] ? path.resolve(process.argv[1]) === fileURLToPath(import.meta.url) : false;
}
