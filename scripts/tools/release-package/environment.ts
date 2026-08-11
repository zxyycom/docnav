import { findCargoExecutable } from "../cargo.ts";
import { isNonArrayRecord } from "../foundation/src/type-guards.ts";
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
  return parseWorkspaceVersion(metadata);
}

export function parseWorkspaceVersion(metadata: unknown): string {
  const { packages, workspaceMembers } = parseCargoMetadata(metadata);
  const packagesById = new Map<string, CargoPackageMetadata>();

  for (const pkg of packages) {
    packagesById.set(pkg.id, pkg);
  }

  const versions = new Set<string>();
  for (const member of workspaceMembers) {
    const pkg = packagesById.get(member);
    if (!pkg) {
      throw new Error(`cargo metadata workspace member ${member} has no matching package`);
    }
    versions.add(pkg.version);
  }

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

function parseCargoMetadata(value: unknown): {
  packages: CargoPackageMetadata[];
  workspaceMembers: string[];
} {
  if (!isNonArrayRecord(value)) {
    throw new Error("cargo metadata root must be an object");
  }

  const workspaceMembers = parseWorkspaceMembers(value.workspace_members);
  const packages = parseCargoPackages(value.packages);
  return { packages, workspaceMembers };
}

function parseWorkspaceMembers(value: unknown): string[] {
  if (!Array.isArray(value)) {
    throw new Error("cargo metadata workspace_members must be an array");
  }

  return value.map((member, index) => {
    if (typeof member !== "string" || member.length === 0) {
      throw new Error(
        `cargo metadata workspace_members[${index}] must be a non-empty string`,
      );
    }
    return member;
  });
}

function parseCargoPackages(value: unknown): CargoPackageMetadata[] {
  if (!Array.isArray(value)) {
    throw new Error("cargo metadata packages must be an array");
  }

  return value.map((pkg, index) => parseCargoPackage(pkg, index));
}

function parseCargoPackage(value: unknown, index: number): CargoPackageMetadata {
  if (!isNonArrayRecord(value)) {
    throw new Error(`cargo metadata packages[${index}] must be an object`);
  }
  if (typeof value.id !== "string" || value.id.length === 0) {
    throw new Error(`cargo metadata packages[${index}].id must be a non-empty string`);
  }
  if (typeof value.version !== "string" || value.version.length === 0) {
    throw new Error(
      `cargo metadata packages[${index}].version must be a non-empty string`,
    );
  }
  return { id: value.id, version: value.version };
}
