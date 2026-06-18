export type ManifestArgs = {
  manifestPath: string | null;
  target: string | null;
  expectProducerKind: "github-actions" | "local" | null;
  expectSourceDirty: boolean | null;
};

export function parseOptionalTarget(args: string[]): string | null {
  let target: string | null = null;

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--target") {
      target = parseTarget(requireOptionValue(args, index, arg));
      index += 1;
      continue;
    }
    if (arg.startsWith("--")) {
      throw new Error(`unknown option ${arg}`);
    }
    throw new Error(`unexpected positional argument ${arg}`);
  }

  return target;
}

export function parseManifestArgs(args: string[]): ManifestArgs {
  const parsed: ManifestArgs = {
    manifestPath: null,
    target: null,
    expectProducerKind: null,
    expectSourceDirty: null,
  };

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (!arg.startsWith("--")) {
      throw new Error(`unexpected positional argument ${arg}`);
    }

    const value = requireOptionValue(args, index, arg);
    switch (arg) {
      case "--manifest":
        parsed.manifestPath = value;
        break;
      case "--target":
        parsed.target = parseTarget(value);
        break;
      case "--expect-producer-kind":
        parsed.expectProducerKind = parseProducerKind(value);
        break;
      case "--expect-source-dirty":
        parsed.expectSourceDirty = parseBoolean(value, "--expect-source-dirty");
        break;
      default:
        throw new Error(`unknown option ${arg}`);
    }
    index += 1;
  }

  if (parsed.manifestPath && parsed.target) {
    throw new Error("--manifest and --target cannot be used together");
  }
  return parsed;
}

function parseTarget(value: string): string {
  if (value.includes("/") || value.includes("\\") || value.includes("..")) {
    throw new Error("--target must be a Rust target triple, not a path");
  }
  return value;
}

function requireOptionValue(args: string[], index: number, option: string): string {
  const value = args[index + 1];
  if (!value || value.startsWith("--")) {
    throw new Error(`${option} requires a value`);
  }
  return value;
}

function parseBoolean(value: string, label: string): boolean {
  if (value === "true") {
    return true;
  }
  if (value === "false") {
    return false;
  }
  throw new Error(`${label} must be true or false`);
}

function parseProducerKind(value: string): "github-actions" | "local" {
  if (value === "local" || value === "github-actions") {
    return value;
  }
  throw new Error("--expect-producer-kind must be local or github-actions");
}
