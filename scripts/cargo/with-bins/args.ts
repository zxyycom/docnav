import {
  booleanOption,
  parseScriptArgs,
  stringArrayOption,
  type ParsedScriptArgs
} from "../../tools/args.ts";
import { commandFromTokens } from "./command.ts";
import { parseBinarySpecs, type BinarySpec } from "./specs.ts";

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
    "usage: node scripts/cargo/with-bins.ts [--quiet] --bin <cargo-package>:<bin-name>:<ENV_NAME> [--bin ...] -- <command> [args...]"
  );
  process.exit(2);
}
