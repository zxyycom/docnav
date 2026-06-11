export function parseOptionalTarget(args) {
  let target = null;

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--target") {
      target = requireOptionValue(args, index, arg);
      if (
        target.includes("/") ||
        target.includes("\\") ||
        target.includes("..")
      ) {
        throw new Error("--target must be a Rust target triple, not a path");
      }
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

export function parseManifestArgs(args) {
  const parsed = {
    manifestPath: null,
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
      case "--expect-producer-kind":
        parsed.expectProducerKind = value;
        break;
      case "--expect-source-dirty":
        parsed.expectSourceDirty = parseBoolean(value, "--expect-source-dirty");
        break;
      default:
        throw new Error(`unknown option ${arg}`);
    }
    index += 1;
  }

  if (!parsed.manifestPath) {
    throw new Error("--manifest is required");
  }
  if (
    parsed.expectProducerKind !== null &&
    parsed.expectProducerKind !== "local" &&
    parsed.expectProducerKind !== "github-actions"
  ) {
    throw new Error("--expect-producer-kind must be local or github-actions");
  }

  return parsed;
}

function requireOptionValue(args, index, option) {
  const value = args[index + 1];
  if (!value || value.startsWith("--")) {
    throw new Error(`${option} requires a value`);
  }
  return value;
}

function parseBoolean(value, label) {
  if (value === "true") {
    return true;
  }
  if (value === "false") {
    return false;
  }
  throw new Error(`${label} must be true or false`);
}
