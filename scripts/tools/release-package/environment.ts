import { findCargoExecutable } from "../cargo.ts";
import { isRecord } from "../types.ts";
import type { ReleaseProducer } from "./config.ts";
import { runCommand } from "./io.ts";

type CargoPackageMetadata = {
  id: string;
  version: string;
};

export function resolveWorkspaceVersion(): string {
  const result = runCommand(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1"],
    {
      label: "cargo metadata",
    },
  );
  const metadata: unknown = JSON.parse(result.stdout);
  assertCargoMetadata(metadata);
  const workspaceMembers = new Set(metadata.workspace_members);
  const versions = new Set(
    metadata.packages
      .filter((pkg) => workspaceMembers.has(pkg.id))
      .map((pkg) => pkg.version),
  );

  if (versions.size !== 1) {
    throw new Error(`expected one workspace version, found ${versions.size}`);
  }

  const version = [...versions][0];
  if (!version) {
    throw new Error("cargo metadata did not report a workspace version");
  }
  return version;
}

export function resolveHostTarget(): string {
  const result = runCommand("rustc", ["-vV"], {
    label: "rustc -vV",
    maxBuffer: 1024 * 1024,
  });
  const hostLine = (result.stdout ?? "")
    .split(/\r?\n/)
    .find((line) => line.startsWith("host: "));

  if (!hostLine) {
    throw new Error("rustc -vV did not report host target");
  }

  return hostLine.slice("host: ".length).trim();
}

export function buildReleaseBinary(packageName: string, binName: string, target: string): string {
  const args = [
    "build",
    "--release",
    "-p",
    packageName,
    "--bin",
    binName,
    "--target",
    target,
    "--message-format=json",
  ];
  const result = runCommand("cargo", args, {
    label: `cargo build --release -p ${packageName} --bin ${binName} --target ${target}`,
  });
  const executable = findCargoExecutable(result.stdout ?? "", binName);

  if (!executable) {
    throw new Error(`cargo build did not report a ${binName} executable`);
  }

  return executable;
}

export function getGitCommit(): string {
  const result = runCommand("git", ["rev-parse", "HEAD"], {
    label: "git rev-parse HEAD",
    maxBuffer: 1024 * 1024,
  });
  return (result.stdout ?? "").trim();
}

export function isSourceDirty(): boolean {
  const result = runCommand(
    "git",
    ["status", "--porcelain=v1", "--untracked-files=all", "--ignored=no"],
    {
      label: "git status --porcelain=v1 --untracked-files=all --ignored=no",
      maxBuffer: 1024 * 1024,
    },
  );
  return (result.stdout ?? "").trim().length > 0;
}

export function resolveProducerMetadata(): ReleaseProducer {
  if (process.env.GITHUB_ACTIONS !== "true") {
    return {
      kind: "local",
      workflow: null,
      run_id: null,
      run_attempt: null,
    };
  }

  return {
    kind: "github-actions",
    workflow: requiredEnv("GITHUB_WORKFLOW"),
    run_id: requiredIntEnv("GITHUB_RUN_ID"),
    run_attempt: requiredIntEnv("GITHUB_RUN_ATTEMPT"),
  };
}

function requiredEnv(name: string): string {
  const value = process.env[name];
  if (!value) {
    throw new Error(`${name} is required`);
  }
  return value;
}

function requiredIntEnv(name: string): number {
  const value = requiredEnv(name);
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(`${name} must be a positive integer`);
  }
  return parsed;
}

function assertCargoMetadata(value: unknown): asserts value is {
  packages: CargoPackageMetadata[];
  workspace_members: string[];
} {
  if (!isRecord(value)) {
    throw new Error("cargo metadata root must be an object");
  }

  const workspaceMembers = Array.isArray(value.workspace_members)
    ? value.workspace_members.filter((member): member is string => typeof member === "string")
    : [];
  const packages = Array.isArray(value.packages)
    ? value.packages.filter(isCargoPackageMetadata)
    : [];

  value.workspace_members = workspaceMembers;
  value.packages = packages;
}

function isCargoPackageMetadata(value: unknown): value is CargoPackageMetadata {
  return isRecord(value) && typeof value.id === "string" && typeof value.version === "string";
}
