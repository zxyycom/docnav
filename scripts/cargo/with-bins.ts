import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildCargoExecutables,
  reportCargoExecutableBuildFailure,
  type CargoBinarySpec
} from "../tools/cargo.ts";
import {
  booleanOption,
  parseScriptArgs,
  stringArrayOption,
  type ParsedScriptArgs,
  type ScriptArgToken
} from "../tools/args.ts";
import { runProcessSync } from "../tools/process.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

type BinarySpec = CargoBinarySpec & {
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

const result = runProcessSync(options.command[0], options.command.slice(1), {
  cwd: root,
  env,
  stdio: "inherit"
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);

function parseArgs(args: string[]): Options {
  let parsed: ParsedScriptArgs;
  try {
    parsed = parseScriptArgs({
      allowPositionals: true,
      args,
      options: {
        bin: { type: "string", multiple: true },
        quiet: { type: "boolean" }
      }
    });
  } catch (error: unknown) {
    usage(error instanceof Error ? error.message : String(error));
  }

  const binaries = stringArrayOption(parsed.values, "bin").map(parseBinarySpec);
  if (binaries.length === 0) {
    usage("at least one --bin declaration is required");
  }
  if (new Set(binaries.map((binary) => binary.binName)).size !== binaries.length) {
    usage("bin names must be unique");
  }
  if (new Set(binaries.map((binary) => binary.envName)).size !== binaries.length) {
    usage("environment variable names must be unique");
  }

  return {
    binaries,
    command: commandFromTokens(parsed.tokens),
    quiet: booleanOption(parsed.values, "quiet")
  };
}

function parseBinarySpec(value: string): BinarySpec {
  const parts = value.split(":");
  if (parts.length !== 3 || parts.some((part) => part.length === 0)) {
    usage(`--bin requires <cargo-package>:<bin-name>:<ENV_NAME>, got ${value}`);
  }

  const [packageName, binName, envName] = parts as [string, string, string];
  if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(envName)) {
    usage(`${envName} must be a valid environment variable name`);
  }
  return { packageName, binName, envName };
}

function commandFromTokens(tokens: readonly ScriptArgToken[]): [string, ...string[]] {
  const separator = tokens.find((token) => token.kind === "option-terminator");
  if (!separator) {
    usage("missing -- command separator");
  }

  const unexpectedPositional = tokens.find(
    (token) => token.kind === "positional" && token.index < separator.index
  );
  if (unexpectedPositional) {
    usage(`unexpected positional argument before --: ${unexpectedPositional.value ?? ""}`);
  }

  const command = tokens
    .filter((token) => token.kind === "positional" && token.index > separator.index)
    .map((token) => token.value)
    .filter((value): value is string => value !== undefined);
  if (command.length === 0) {
    usage("missing command after --");
  }

  return command as [string, ...string[]];
}

function buildCargoBins(binaries: BinarySpec[], quiet: boolean): Map<string, string> {
  const result = buildCargoExecutables({ binaries, cwd: root });

  if (!result.ok) {
    process.exit(reportCargoExecutableBuildFailure(result));
  }

  if (result.stderr && !quiet) {
    process.stderr.write(result.stderr);
  }

  return result.executables;
}

function usage(message: string): never {
  console.error(message);
  console.error(
    "usage: node scripts/cargo/with-bins.ts [--quiet] --bin <cargo-package>:<bin-name>:<ENV_NAME> [--bin ...] -- <command> [args...]"
  );
  process.exit(2);
}
