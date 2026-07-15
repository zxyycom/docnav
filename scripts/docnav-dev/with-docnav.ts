import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildCargoExecutables,
  reportCargoExecutableBuildFailure
} from "../tools/cargo.ts";
import { runProcessSync } from "../tools/foundation/src/process.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const docnavBinary = { packageName: "docnav", binName: "docnav" };

const options = parseArgs(process.argv.slice(2));
const executable = buildDocnavExecutable(options.quiet);
const result = runProcessSync(options.command[0], options.command.slice(1), {
  cwd: root,
  env: { ...process.env, DOCNAV_BIN: executable },
  stdio: "inherit"
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);

function parseArgs(args: string[]): {
  command: [string, ...string[]];
  quiet: boolean;
} {
  const quiet = args[0] === "--quiet";
  if (!quiet && args[0]?.startsWith("-")) {
    usage(`unknown option: ${args[0]}`);
  }

  const command = args.slice(quiet ? 1 : 0);
  if (command.length === 0) {
    usage("missing command");
  }

  return {
    command: command as [string, ...string[]],
    quiet
  };
}

function buildDocnavExecutable(quiet: boolean): string {
  const result = buildCargoExecutables({ binaries: [docnavBinary], cwd: root });

  if (!result.ok) {
    process.exit(reportCargoExecutableBuildFailure(result));
  }

  if (result.stderr && !quiet) {
    process.stderr.write(result.stderr);
  }

  const executable = result.executables.get(docnavBinary.binName);
  if (!executable) {
    console.error("cargo build did not report a docnav executable");
    process.exit(1);
  }
  return executable;
}

function usage(message: string): never {
  console.error(message);
  console.error(
    "usage: bun scripts/docnav-dev/with-docnav.ts [--quiet] <command> [args...]"
  );
  process.exit(2);
}
