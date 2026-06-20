import { parseScriptArgs, stringOption } from "../args.ts";
import {
  normalizeParseArgsError,
  parseOptionalBoolean,
  parseOptionalProducerKind,
  parseOptionalTargetValue,
  type ProducerKind
} from "./args/values.ts";

export type ManifestArgs = {
  manifestPath: string | null;
  target: string | null;
  expectProducerKind: ProducerKind | null;
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
    throw new Error(normalizeParseArgsError(error), { cause: error });
  }
}

function rejectPositionals(positionals: readonly string[]): void {
  if (positionals.length > 0) {
    throw new Error(`unexpected positional argument ${positionals[0]}`);
  }
}
