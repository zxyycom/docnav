import { findCargoExecutable } from "../cargo.ts";
import { runCommand } from "./io.ts";

export function resolveWorkspaceVersion() {
  const result = runCommand(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1"],
    {
      label: "cargo metadata",
    },
  );
  const metadata = JSON.parse(result.stdout);
  const workspaceMembers = new Set(metadata.workspace_members ?? []);
  const versions = new Set(
    (metadata.packages ?? [])
      .filter((pkg: ExternalValue) => workspaceMembers.has(pkg.id))
      .map((pkg: ExternalValue) => pkg.version),
  );

  if (versions.size !== 1) {
    throw new Error(`expected one workspace version, found ${versions.size}`);
  }

  return [...versions][0];
}

export function resolveHostTarget() {
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

export function buildReleaseBinary(packageName: ExternalValue, binName: ExternalValue, target: ExternalValue) {
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

export function getGitCommit() {
  const result = runCommand("git", ["rev-parse", "HEAD"], {
    label: "git rev-parse HEAD",
    maxBuffer: 1024 * 1024,
  });
  return (result.stdout ?? "").trim();
}

export function isSourceDirty() {
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

export function resolveProducerMetadata() {
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

function requiredEnv(name: ExternalValue) {
  const value = process.env[name];
  if (!value) {
    throw new Error(`${name} is required`);
  }
  return value;
}

function requiredIntEnv(name: ExternalValue) {
  const value = requiredEnv(name);
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(`${name} must be a positive integer`);
  }
  return parsed;
}
