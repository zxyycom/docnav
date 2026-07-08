import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildCargoExecutables,
  reportCargoExecutableBuildFailure
} from "../tools/cargo.ts";
import { runProcessSync } from "../tools/foundation/src/process.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const TOML_SECTION_HEADER_PATTERN = new RegExp("^\\[[^\\]]+\\]$");
const TOML_VERSION_ASSIGNMENT_PATTERN = new RegExp("^version\\s*=\\s*\"([^\"]+)\"");

type BinaryCandidate = {
  binaryPath: string;
  mtimeMs: number;
  score: number;
};

const binaryPath = resolveDocnavBinary();
const result = runProcessSync(binaryPath, process.argv.slice(2), {
  cwd: process.cwd(),
  env: process.env,
  stdio: "inherit"
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);

function resolveDocnavBinary(): string {
  const packagedBinary = findPackagedDocnavBinary();
  if (packagedBinary) {
    return packagedBinary;
  }

  const debugBinary = path.join(root, "target", "debug", docnavBinaryName());
  if (fs.existsSync(debugBinary)) {
    return debugBinary;
  }

  return buildDebugDocnavBinary();
}

function findPackagedDocnavBinary(): string | null {
  const artifactsRoot = path.join(root, "artifacts", "docnav");
  const candidates = collectPackagedBinaryCandidates(artifactsRoot);
  if (candidates.length === 0) {
    return null;
  }

  candidates.sort((left, right) => {
    if (left.score !== right.score) {
      return right.score - left.score;
    }
    if (left.mtimeMs !== right.mtimeMs) {
      return right.mtimeMs - left.mtimeMs;
    }
    return left.binaryPath.localeCompare(right.binaryPath);
  });

  return candidates[0]?.binaryPath ?? null;
}

function collectPackagedBinaryCandidates(artifactsRoot: string): BinaryCandidate[] {
  const binaryName = docnavBinaryName();
  const workspaceVersion = readWorkspaceVersion();
  const hostHints = hostTargetHints();
  const candidates: BinaryCandidate[] = [];

  visit(artifactsRoot);
  return candidates;

  function visit(directory: string): void {
    let entries: fs.Dirent[];
    try {
      entries = fs.readdirSync(directory, { withFileTypes: true });
    } catch {
      return;
    }

    for (const entry of entries) {
      const entryPath = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        visit(entryPath);
        continue;
      }

      if (!entry.isFile() || entry.name.toLowerCase() !== binaryName.toLowerCase()) {
        continue;
      }

      if (path.basename(path.dirname(entryPath)).toLowerCase() !== "package") {
        continue;
      }

      const stats = fs.statSync(entryPath);
      candidates.push({
        binaryPath: entryPath,
        mtimeMs: stats.mtimeMs,
        score: candidateScore(entryPath, workspaceVersion, hostHints)
      });
    }
  }
}

function candidateScore(
  binaryPath: string,
  workspaceVersion: string | null,
  hostHints: readonly string[]
): number {
  const pathSegments = binaryPath.split(path.sep);
  const normalizedPath = binaryPath.toLowerCase();
  let score = 0;

  if (workspaceVersion && pathSegments.includes(`v${workspaceVersion}`)) {
    score += 100;
  }

  for (const hint of hostHints) {
    if (normalizedPath.includes(hint)) {
      score += 10;
    }
  }

  return score;
}

function buildDebugDocnavBinary(): string {
  console.error("docnav package binary was not found; building target/debug/docnav.");
  const buildResult = buildCargoExecutables({
    binaries: [{ packageName: "docnav", binName: "docnav" }],
    cwd: root
  });

  if (!buildResult.ok) {
    process.exit(reportCargoExecutableBuildFailure(buildResult));
  }

  if (buildResult.stderr) {
    process.stderr.write(buildResult.stderr);
  }

  const executable = buildResult.executables.get("docnav");
  if (!executable) {
    console.error("cargo build did not report a docnav executable");
    process.exit(1);
  }

  return executable;
}

function readWorkspaceVersion(): string | null {
  const content = readWorkspaceCargoToml();
  return content === null ? null : workspacePackageVersion(content);
}

function readWorkspaceCargoToml(): string | null {
  try {
    return fs.readFileSync(path.join(root, "Cargo.toml"), "utf8");
  } catch {
    return null;
  }
}

function workspacePackageVersion(content: string): string | null {
  let inWorkspacePackage = false;
  for (const line of content.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (isTomlSectionHeader(trimmed)) {
      inWorkspacePackage = trimmed === "[workspace.package]";
      continue;
    }

    if (!inWorkspacePackage) {
      continue;
    }

    const version = workspaceVersionAssignment(trimmed);
    if (version !== null) {
      return version;
    }
  }

  return null;
}

function isTomlSectionHeader(line: string): boolean {
  return TOML_SECTION_HEADER_PATTERN.test(line);
}

function workspaceVersionAssignment(line: string): string | null {
  const versionMatch = TOML_VERSION_ASSIGNMENT_PATTERN.exec(line);
  if (!versionMatch) {
    return null;
  }

  return versionMatch[1] || null;
}

function hostTargetHints(): string[] {
  const hints = [
    platformTargetHint(),
    archTargetHint()
  ].filter((hint): hint is string => hint !== null);

  return hints.map((hint) => hint.toLowerCase());
}

function platformTargetHint(): string | null {
  if (process.platform === "win32") {
    return "windows";
  }
  if (process.platform === "darwin") {
    return "apple-darwin";
  }
  if (process.platform === "linux") {
    return "linux";
  }
  return null;
}

function archTargetHint(): string | null {
  if (process.arch === "x64") {
    return "x86_64";
  }
  if (process.arch === "arm64") {
    return "aarch64";
  }
  return null;
}

function docnavBinaryName(): string {
  return process.platform === "win32" ? "docnav.exe" : "docnav";
}
