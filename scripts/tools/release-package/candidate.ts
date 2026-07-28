import fs from "node:fs";
import path from "node:path";

import {
  compareStrings,
  publicBinaryName,
  type ReleaseProducer,
} from "./config.ts";
import {
  getGitCommit,
  isSourceDirty,
  resolveProducerMetadata,
  resolveWorkspaceVersion,
} from "./environment.ts";
import { runCommand, sha256File } from "./io.ts";
import { assert, assertEqualLists } from "./validation/assertions.ts";
import { validateReleasePackage } from "./validation.ts";

const supportedTargets = [
  "x86_64-unknown-linux-gnu",
  "x86_64-pc-windows-msvc",
] as const;
type SupportedTarget = (typeof supportedTargets)[number];
type GitHubActionsProducer = Extract<
  ReleaseProducer,
  { kind: "github-actions" }
>;

export type CandidateExpectations = {
  gitCommit: string;
  producer: GitHubActionsProducer;
  sourceDirty: boolean;
  tag: {
    gitCommit: string;
    name: string;
  } | null;
  version: string;
};

type CandidateTargetResult = {
  manifestHash: string;
  packageBinaryHash: string;
  publicBinaryHash: string;
  target: SupportedTarget;
};

type ValidatedTargetPackage = {
  manifestHash: string;
  packageDir: string;
  coreEntry: { path: string; sha256: string };
};

type PublicFileSet = { binaryName: string; checksumName: string };

export type CandidateValidationResult = {
  candidateRoot: string;
  gitCommit: string;
  targets: CandidateTargetResult[];
  version: string;
};

export function resolveCandidateExpectations(
  tagName: string | null,
): CandidateExpectations {
  const version = resolveWorkspaceVersion();
  const gitCommit = getGitCommit();
  const workflowCommit = requiredEnv("GITHUB_SHA");
  assert(
    gitCommit === workflowCommit,
    "workspace HEAD must match current workflow commit",
  );

  const producer = resolveProducerMetadata();
  assert(
    producer.kind === "github-actions",
    "candidate validation requires GitHub Actions producer metadata",
  );

  return {
    gitCommit,
    producer,
    sourceDirty: isSourceDirty(),
    tag: tagName
      ? {
          gitCommit: resolveTagCommit(tagName, version),
          name: tagName,
        }
      : null,
    version,
  };
}

export function validateReleaseCandidate(
  candidateRoot: string,
  expectations: CandidateExpectations,
): CandidateValidationResult {
  const resolvedCandidateRoot = path.resolve(candidateRoot);
  assert(
    path.basename(resolvedCandidateRoot) === `v${expectations.version}`,
    `candidate version root must be v${expectations.version}`,
  );
  assert(!expectations.sourceDirty, "workspace source must be clean");
  validateTag(expectations);

  const targetEntries = readDirectory(
    resolvedCandidateRoot,
    "candidate version root",
  );
  const targetNames = targetEntries
    .map((entry) => entry.name)
    .sort(compareStrings);
  assertEqualLists(
    targetNames,
    [...supportedTargets].sort(compareStrings),
    "candidate must contain exactly the supported target directories",
  );
  assert(
    targetEntries.every((entry) => entry.isDirectory()),
    "candidate target entries must be directories",
  );

  return {
    candidateRoot: resolvedCandidateRoot,
    gitCommit: expectations.gitCommit,
    targets: supportedTargets.map((target) =>
      validateTarget(resolvedCandidateRoot, target, expectations),
    ),
    version: expectations.version,
  };
}

function validateTarget(
  candidateRoot: string,
  target: SupportedTarget,
  expectations: CandidateExpectations,
): CandidateTargetResult {
  const targetRoot = path.join(candidateRoot, target);
  const packageDir = path.join(targetRoot, "package");
  const publicDir = path.join(targetRoot, "public");
  assertDirectory(packageDir, `${target} package`);
  assertDirectory(publicDir, `${target} public`);

  const validatedPackage = validateTargetPackage(
    packageDir,
    target,
    expectations,
  );
  const publicFiles = validatePublicFileSet(
    publicDir,
    target,
    expectations.version,
  );
  const publicBinaryHash = validatePublicBinaryParity(
    validatedPackage,
    publicDir,
    publicFiles,
  );
  return {
    manifestHash: validatedPackage.manifestHash,
    packageBinaryHash: validatedPackage.coreEntry.sha256,
    publicBinaryHash,
    target,
  };
}

function validateTargetPackage(
  packageDir: string,
  target: SupportedTarget,
  expectations: CandidateExpectations,
): ValidatedTargetPackage {
  const validatedPackage = validateReleasePackage(
    path.join(packageDir, "manifest.json"),
    {
      expectProducerKind: "github-actions",
      expectSourceDirty: false,
    },
  );
  const { manifest } = validatedPackage;
  assert(
    manifest.version === expectations.version,
    "manifest.version must match workspace version",
  );
  assert(
    manifest.target === target,
    "manifest.target must match candidate target",
  );
  assert(
    manifest.git_commit === expectations.gitCommit,
    "manifest.git_commit must match candidate commit",
  );
  assert(
    manifest.producer.kind === "github-actions" &&
      manifest.producer.workflow === expectations.producer.workflow &&
      manifest.producer.run_id === expectations.producer.run_id &&
      manifest.producer.run_attempt === expectations.producer.run_attempt,
    "manifest producer must match current workflow run",
  );

  const coreEntry = validatedPackage.fileEntries.find(
    (entry) => entry.component === "core",
  );
  assert(
    coreEntry &&
      typeof coreEntry.path === "string" &&
      typeof coreEntry.sha256 === "string",
    "validated manifest must contain a core binary entry",
  );

  return {
    manifestHash: validatedPackage.manifestHash,
    packageDir: validatedPackage.packageDir,
    coreEntry: { path: coreEntry.path, sha256: coreEntry.sha256 },
  };
}

function validatePublicFileSet(
  publicDir: string,
  target: SupportedTarget,
  version: string,
): PublicFileSet {
  const binaryName = publicBinaryName(version, target);
  const checksumName = `${binaryName}.sha256`;
  const publicEntries = readDirectory(publicDir, `${target} public`);
  const publicNames = publicEntries
    .map((entry) => entry.name)
    .sort(compareStrings);
  assertEqualLists(
    publicNames,
    [binaryName, checksumName].sort(compareStrings),
    "public directory must contain exactly the binary and checksum",
  );
  assert(
    publicEntries.every((entry) => entry.isFile()),
    "public directory entries must be files",
  );
  return { binaryName, checksumName };
}

function validatePublicBinaryParity(
  validatedPackage: ValidatedTargetPackage,
  publicDir: string,
  publicFiles: PublicFileSet,
): string {
  const packageBinaryPath = path.join(
    validatedPackage.packageDir,
    validatedPackage.coreEntry.path,
  );
  const publicBinaryPath = path.join(publicDir, publicFiles.binaryName);
  const publicBinaryHash = sha256File(publicBinaryPath);
  assert(
    publicBinaryHash === validatedPackage.coreEntry.sha256,
    "public binary sha256 must match canonical package binary",
  );
  assert(
    fs
      .readFileSync(publicBinaryPath)
      .equals(fs.readFileSync(packageBinaryPath)),
    "public binary bytes must match canonical package binary",
  );

  const checksumPath = path.join(publicDir, publicFiles.checksumName);
  assert(
    fs.readFileSync(checksumPath, "utf8") ===
      `${publicBinaryHash}  ${publicFiles.binaryName}\n`,
    "public checksum must match binary hash and filename",
  );
  return publicBinaryHash;
}

function validateTag(expectations: CandidateExpectations): void {
  if (!expectations.tag) {
    return;
  }
  assertTagName(expectations.tag.name, expectations.version);
  assert(
    expectations.tag.gitCommit === expectations.gitCommit,
    "tag commit must match candidate commit",
  );
}

function resolveTagCommit(tagName: string, version: string): string {
  assertTagName(tagName, version);
  const result = runCommand(
    "git",
    ["rev-parse", "--verify", `refs/tags/${tagName}^{commit}`],
    {
      label: `git resolve tag ${tagName}`,
      maxBuffer: 1024 * 1024,
    },
  );
  const gitCommit = (result.stdout ?? "").trim();
  assert(gitCommit.length > 0, `tag ${tagName} did not resolve to a commit`);
  return gitCommit;
}

function assertTagName(tagName: string, version: string): void {
  assert(tagName === `v${version}`, `candidate tag must be v${version}`);
}

function requiredEnv(name: string): string {
  const value = process.env[name];
  assert(
    typeof value === "string" && value.length > 0,
    `${name} is required`,
  );
  return value;
}

function readDirectory(directory: string, label: string): fs.Dirent[] {
  assertDirectory(directory, label);
  return fs.readdirSync(directory, { withFileTypes: true });
}

function assertDirectory(directory: string, label: string): void {
  assert(fs.existsSync(directory), `${label} directory is missing: ${directory}`);
  assert(
    fs.statSync(directory).isDirectory(),
    `${label} must be a directory: ${directory}`,
  );
}
