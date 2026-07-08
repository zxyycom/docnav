import {
  booleanOption,
  parseScriptArgs,
  stringArrayOption,
  type ParsedScriptArgs,
  type ScriptArgToken
} from "../../tools/foundation/src/args.ts";
import type { CargoBinarySpec } from "../../tools/cargo.ts";

export type BinarySpec = CargoBinarySpec & {
  envName: string;
};

export type WithBinsOptions = {
  binaries: BinarySpec[];
  command: [string, ...string[]];
  quiet: boolean;
};

export function parseWithBinsArgs(args: string[]): WithBinsOptions {
  try {
    const parsed = parseArgs(args);
    const binaries = parseBinarySpecs(stringArrayOption(parsed.values, "bin"));

    return {
      binaries,
      command: commandFromTokens(parsed.tokens),
      quiet: booleanOption(parsed.values, "quiet")
    };
  } catch (error: unknown) {
    usage(error instanceof Error ? error.message : String(error));
  }
}

function parseArgs(args: string[]): ParsedScriptArgs {
  return parseScriptArgs({
    allowPositionals: true,
    args,
    options: {
      bin: { type: "string", multiple: true },
      quiet: { type: "boolean" }
    }
  });
}

function usage(message: string): never {
  console.error(message);
  console.error(
    "usage: bun scripts/cargo/with-bins.ts [--quiet] --bin <cargo-package>:<bin-name>:<ENV_NAME> [--bin ...] -- <command> [args...]"
  );
  process.exit(2);
}

function commandFromTokens(tokens: readonly ScriptArgToken[]): [string, ...string[]] {
  const separator = tokens.find((token) => token.kind === "option-terminator");
  if (!separator) {
    throw new Error("missing -- command separator");
  }

  const unexpectedPositional = tokens.find(
    (token) => token.kind === "positional" && token.index < separator.index
  );
  if (unexpectedPositional) {
    throw new Error(`unexpected positional argument before --: ${unexpectedPositional.value ?? ""}`);
  }

  const command = tokens
    .filter((token) => token.kind === "positional" && token.index > separator.index)
    .map((token) => token.value)
    .filter((value): value is string => value !== undefined);
  if (command.length === 0) {
    throw new Error("missing command after --");
  }

  return command as [string, ...string[]];
}

function parseBinarySpecs(values: string[]): BinarySpec[] {
  const binaries = values.map(parseBinarySpec);
  validateBinarySpecs(binaries);
  return binaries;
}

function validateBinarySpecs(binaries: readonly BinarySpec[]): void {
  if (binaries.length === 0) {
    throw new Error("at least one --bin declaration is required");
  }
  if (new Set(binaries.map((binary) => binary.binName)).size !== binaries.length) {
    throw new Error("bin names must be unique");
  }
  if (new Set(binaries.map((binary) => binary.envName)).size !== binaries.length) {
    throw new Error("environment variable names must be unique");
  }
}

function parseBinarySpec(value: string): BinarySpec {
  const parts = value.split(":");
  if (parts.length !== 3 || parts.some((part) => part.length === 0)) {
    throw new Error(`--bin requires <cargo-package>:<bin-name>:<ENV_NAME>, got ${value}`);
  }

  const [packageName, binName, envName] = parts as [string, string, string];
  if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(envName)) {
    throw new Error(`${envName} must be a valid environment variable name`);
  }
  return { packageName, binName, envName };
}
