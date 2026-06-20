import type { CargoBinarySpec } from "../../tools/cargo.ts";

export type BinarySpec = CargoBinarySpec & {
  envName: string;
};

export function parseBinarySpecs(values: string[]): BinarySpec[] {
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
