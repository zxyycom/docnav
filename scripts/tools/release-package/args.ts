import { parseScriptArgs, stringOption } from "../args.ts";
import { errorMessage } from "../errors.ts";

export type ManifestArgs = {
  manifestPath: string | null;
  target: string | null;
  expectProducerKind: "github-actions" | "local" | null;
  expectSourceDirty: boolean | null;
};

export function parseOptionalTarget(args: string[]): string | null {
  const parsed = parseReleaseArgs(args, {
    target: { type: "string" }
  });
  rejectPositionals(parsed.positionals);
  return parseOptionalTargetValue(stringOption(parsed.values, "target"));
}

export function parseManifestArgs(args: string[]): ManifestArgs {
  const parsed = parseReleaseArgs(args, {
    manifest: { type: "string" },
    target: { type: "string" },
    "expect-producer-kind": { type: "string" },
    "expect-source-dirty": { type: "string" }
  });
  rejectPositionals(parsed.positionals);

  const manifestPath = stringOption(parsed.values, "manifest") ?? null;
  const target = parseOptionalTargetValue(stringOption(parsed.values, "target"));
  const expectProducerKind = parseOptionalProducerKind(stringOption(parsed.values, "expect-producer-kind"));
  const expectSourceDirty = parseOptionalBoolean(stringOption(parsed.values, "expect-source-dirty"), "--expect-source-dirty");

  if (manifestPath && target) {
    throw new Error("--manifest and --target cannot be used together");
  }
  return {
    expectProducerKind,
    expectSourceDirty,
    manifestPath,
    target
  };
}

function parseTarget(value: string): string {
  if (value.includes("/") || value.includes("\\") || value.includes("..")) {
    throw new Error("--target must be a Rust target triple, not a path");
  }
  return value;
}

function parseOptionalTargetValue(value: string | undefined): string | null {
  return value === undefined ? null : parseTarget(value);
}

function parseOptionalBoolean(value: string | undefined, label: string): boolean | null {
  if (value === undefined) {
    return null;
  }
  if (value === "true") {
    return true;
  }
  if (value === "false") {
    return false;
  }
  throw new Error(`${label} must be true or false`);
}

function parseOptionalProducerKind(value: string | undefined): "github-actions" | "local" | null {
  if (value === undefined) {
    return null;
  }
  if (value === "local" || value === "github-actions") {
    return value;
  }
  throw new Error("--expect-producer-kind must be local or github-actions");
}

function parseReleaseArgs(
  args: string[],
  options: Parameters<typeof parseScriptArgs>[0]["options"]
): ReturnType<typeof parseScriptArgs> {
  try {
    return parseScriptArgs({
      allowPositionals: true,
      args,
      options
    });
  } catch (error: unknown) {
    throw new Error(normalizeParseArgsError(errorMessage(error)), { cause: error });
  }
}

function rejectPositionals(positionals: readonly string[]): void {
  if (positionals.length > 0) {
    throw new Error(`unexpected positional argument ${positionals[0]}`);
  }
}

function normalizeParseArgsError(message: string): string {
  const unknownOption = message.match(/^Unknown option '(--[^']+)'/);
  if (unknownOption) {
    return `unknown option ${unknownOption[1]}`;
  }

  const missingOptionValue = message.match(/^Option '(--[^ ]+) <value>' argument missing$/);
  if (missingOptionValue) {
    return `${missingOptionValue[1]} requires a value`;
  }

  const unexpectedArgument = message.match(/^Unexpected argument '([^']+)'/);
  if (unexpectedArgument) {
    return `unexpected positional argument ${unexpectedArgument[1]}`;
  }

  return message;
}
