import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
export const artifactsRoot = path.join(root, "artifacts", "docnav");

export const releaseComponents = Object.freeze([
  Object.freeze({
    component: "core",
    packageName: "docnav",
    binName: "docnav"
  }),
  Object.freeze({
    component: "adapter",
    adapterId: "docnav-markdown",
    packageName: "docnav-markdown",
    binName: "docnav-markdown"
  })
]);

export function parseOptionalTarget(args) {
  let target = null;
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--target") {
      const value = args[index + 1];
      if (!value || value.startsWith("--")) {
        throw new Error("--target requires a value");
      }
      if (value.includes("/") || value.includes("\\") || value.includes("..")) {
        throw new Error("--target must be a Rust target triple, not a path");
      }
      target = value;
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
  let manifestPath = null;
  let expectProducerKind = null;
  let expectSourceDirty = null;

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (!arg.startsWith("--")) {
      throw new Error(`unexpected positional argument ${arg}`);
    }
    const value = args[index + 1];
    if (!value || value.startsWith("--")) {
      throw new Error(`${arg} requires a value`);
    }
    switch (arg) {
      case "--manifest":
        manifestPath = value;
        break;
      case "--expect-producer-kind":
        expectProducerKind = value;
        break;
      case "--expect-source-dirty":
        expectSourceDirty = parseBoolean(value, "--expect-source-dirty");
        break;
      default:
        throw new Error(`unknown option ${arg}`);
    }
    index += 1;
  }

  if (!manifestPath) {
    throw new Error("--manifest is required");
  }

  if (expectProducerKind !== null && expectProducerKind !== "local" && expectProducerKind !== "github-actions") {
    throw new Error("--expect-producer-kind must be local or github-actions");
  }

  return {
    manifestPath,
    expectProducerKind,
    expectSourceDirty
  };
}

export function resolveWorkspaceVersion() {
  const result = spawnSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
    cwd: root,
    encoding: "utf8",
    windowsHide: true,
    maxBuffer: 1024 * 1024 * 64
  });

  if (result.error || result.status !== 0) {
    throw new Error(composeSpawnError("cargo metadata", result));
  }

  const metadata = JSON.parse(result.stdout);
  const workspaceMembers = new Set(metadata.workspace_members ?? []);
  const versions = new Set(
    (metadata.packages ?? [])
      .filter((pkg) => workspaceMembers.has(pkg.id))
      .map((pkg) => pkg.version)
  );

  if (versions.size !== 1) {
    throw new Error(`expected one workspace version, found ${versions.size}`);
  }

  return [...versions][0];
}

export function resolveHostTarget() {
  const result = spawnSync("rustc", ["-vV"], {
    cwd: root,
    encoding: "utf8",
    windowsHide: true,
    maxBuffer: 1024 * 1024
  });

  if (result.error || result.status !== 0) {
    throw new Error(composeSpawnError("rustc -vV", result));
  }

  const hostLine = (result.stdout ?? "")
    .split(/\r?\n/)
    .find((line) => line.startsWith("host: "));
  if (!hostLine) {
    throw new Error("rustc -vV did not report host target");
  }

  return hostLine.slice("host: ".length).trim();
}

export function resolvePackageLayout(version, target) {
  const packageDir = path.join(artifactsRoot, `v${version}`, target, "package");
  return {
    version,
    target,
    releaseRoot: path.dirname(packageDir),
    packageDir,
    manifestPath: path.join(packageDir, "manifest.json"),
    checksumsPath: path.join(packageDir, "SHA256SUMS.txt")
  };
}

export function resolveBinaryDestPath(packageDir, executablePath) {
  return path.join(packageDir, path.basename(executablePath));
}

export function buildReleaseBinary(packageName, binName, target) {
  const result = spawnSync(
    "cargo",
    ["build", "--release", "-p", packageName, "--bin", binName, "--target", target, "--message-format=json"],
    {
      cwd: root,
      encoding: "utf8",
      windowsHide: true,
      maxBuffer: 1024 * 1024 * 64
    }
  );

  if (result.error || result.status !== 0) {
    throw new Error(composeSpawnError(`cargo build --release -p ${packageName} --bin ${binName} --target ${target}`, result));
  }

  const executable = findCargoExecutable(result.stdout ?? "", binName);
  if (!executable) {
    throw new Error(`cargo build did not report a ${binName} executable`);
  }

  return executable;
}

export function copyExecutable(sourcePath, destPath) {
  fs.mkdirSync(path.dirname(destPath), { recursive: true });
  fs.copyFileSync(sourcePath, destPath);
  const sourceStat = fs.statSync(sourcePath);
  fs.chmodSync(destPath, sourceStat.mode);
}

export function readJsonFile(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

export function writeJsonFile(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

export function writeTextFile(filePath, content) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content, "utf8");
}

export function readTextFile(filePath) {
  return fs.readFileSync(filePath, "utf8");
}

export function sha256File(filePath) {
  const hash = crypto.createHash("sha256");
  hash.update(fs.readFileSync(filePath));
  return hash.digest("hex");
}

export function normalizeRelativePath(filePath) {
  return filePath.replaceAll(path.sep, "/");
}

export function getGitCommit() {
  const result = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8",
    windowsHide: true,
    maxBuffer: 1024 * 1024
  });

  if (result.error || result.status !== 0) {
    throw new Error(composeSpawnError("git rev-parse HEAD", result));
  }

  return (result.stdout ?? "").trim();
}

export function isSourceDirty() {
  const result = spawnSync("git", ["status", "--porcelain=v1", "--untracked-files=all", "--ignored=no"], {
    cwd: root,
    encoding: "utf8",
    windowsHide: true,
    maxBuffer: 1024 * 1024
  });

  if (result.error || result.status !== 0) {
    throw new Error(composeSpawnError("git status --porcelain=v1 --untracked-files=all --ignored=no", result));
  }

  return (result.stdout ?? "").trim().length > 0;
}

export function resolveProducerMetadata() {
  if (process.env.GITHUB_ACTIONS === "true") {
    const workflow = requiredEnv("GITHUB_WORKFLOW");
    const runId = requiredIntEnv("GITHUB_RUN_ID");
    const runAttempt = requiredIntEnv("GITHUB_RUN_ATTEMPT");
    return {
      kind: "github-actions",
      workflow,
      run_id: runId,
      run_attempt: runAttempt
    };
  }

  return {
    kind: "local",
    workflow: null,
    run_id: null,
    run_attempt: null
  };
}

export function resolveReleaseManifest(manifestPath) {
  const packageDir = path.dirname(path.resolve(manifestPath));
  const manifest = readJsonFile(manifestPath);
  return {
    packageDir,
    manifest
  };
}

export function validateReleasePackage(manifestPath, options = {}) {
  const resolvedManifestPath = path.resolve(manifestPath);
  const packageDir = path.dirname(resolvedManifestPath);
  const manifest = readJsonFile(resolvedManifestPath);
  const expectedProducerKind = options.expectProducerKind ?? null;
  const expectedSourceDirty = options.expectSourceDirty ?? null;

  assert(path.basename(resolvedManifestPath) === "manifest.json", "manifest path must end with manifest.json");
  assert(path.basename(packageDir) === "package", "manifest must live in a package/ directory");
  assert(path.basename(path.dirname(packageDir)) === manifest.target, "package target directory must match manifest target");
  assert(path.basename(path.dirname(path.dirname(packageDir))) === `v${manifest.version}`, "package version directory must match manifest version");
  assert(path.basename(path.dirname(path.dirname(path.dirname(packageDir)))) === "docnav", "package root must be artifacts/docnav");

  assert(manifest.schema_version === 1, "manifest.schema_version must be 1");
  assert(manifest.product === "docnav", "manifest.product must be docnav");
  assert(typeof manifest.version === "string" && manifest.version.length > 0, "manifest.version must be a string");
  assert(typeof manifest.target === "string" && manifest.target.length > 0, "manifest.target must be a string");
  assert(typeof manifest.generated_at === "string" && manifest.generated_at.length > 0, "manifest.generated_at must be a string");
  assert(typeof manifest.git_commit === "string" && manifest.git_commit.length > 0, "manifest.git_commit must be a string");
  assert(typeof manifest.source_dirty === "boolean", "manifest.source_dirty must be a boolean");
  assert(Array.isArray(manifest.files), "manifest.files must be an array");
  assert(manifest.files.length === releaseComponents.length, "manifest.files must list all release binaries");

  const producer = manifest.producer ?? {};
  assert(producer.kind === "local" || producer.kind === "github-actions", "manifest.producer.kind must be local or github-actions");

  if (producer.kind === "local") {
    assert(producer.workflow === null, "local producer.workflow must be null");
    assert(producer.run_id === null, "local producer.run_id must be null");
    assert(producer.run_attempt === null, "local producer.run_attempt must be null");
  } else {
    assert(typeof producer.workflow === "string" && producer.workflow.length > 0, "github-actions producer.workflow must be present");
    assert(Number.isInteger(producer.run_id) && producer.run_id > 0, "github-actions producer.run_id must be a positive integer");
    assert(Number.isInteger(producer.run_attempt) && producer.run_attempt > 0, "github-actions producer.run_attempt must be a positive integer");
  }

  if (expectedProducerKind) {
    assert(
      producer.kind === expectedProducerKind,
      `manifest.producer.kind must be ${expectedProducerKind}`
    );
  }

  if (expectedSourceDirty !== null) {
    assert(manifest.source_dirty === expectedSourceDirty, `manifest.source_dirty must be ${expectedSourceDirty}`);
  }

  const expectedFileNames = releaseComponents.map((component) => expectedBinaryName(component.binName, manifest.target));
  const actualFiles = manifest.files.map((entry) => entry.path);
  const expectedSortedFiles = [...expectedFileNames].sort(compareStrings);
  const actualSortedFiles = [...actualFiles].sort(compareStrings);
  assert(
    actualSortedFiles.length === expectedSortedFiles.length &&
      actualSortedFiles.every((value, index) => value === expectedSortedFiles[index]),
    `manifest.files must list exactly ${expectedSortedFiles.join(", ")}`
  );

  const manifestFilesByPath = new Map();
  for (const entry of manifest.files) {
    assert(typeof entry.path === "string" && entry.path.length > 0, "manifest file path must be a string");
    assert(!entry.path.includes("/") && !entry.path.includes("\\"), "manifest file path must be a file name");
    assert(entry.component === "core" || entry.component === "adapter", "manifest file component must be core or adapter");
    assert(Number.isInteger(entry.size_bytes) && entry.size_bytes >= 0, "manifest file size_bytes must be an integer");
    assert(typeof entry.sha256 === "string" && /^[0-9a-f]{64}$/.test(entry.sha256), "manifest file sha256 must be lowercase hex");
    if (entry.component === "adapter") {
      assert(entry.adapter_id === "docnav-markdown", "adapter manifest entry must include adapter_id");
    } else {
      assert(entry.adapter_id === undefined, "core manifest entry must not include adapter_id");
    }
    manifestFilesByPath.set(entry.path, entry);
  }

  const packageEntries = fs.readdirSync(packageDir).sort(compareStrings);
  const expectedPackageEntries = [...expectedFileNames, "manifest.json", "SHA256SUMS.txt"].sort(compareStrings);
  assert(
    packageEntries.length === expectedPackageEntries.length &&
      packageEntries.every((value, index) => value === expectedPackageEntries[index]),
    `package directory must contain exactly ${expectedPackageEntries.join(", ")}`
  );

  for (const fileName of expectedFileNames) {
    const entry = manifestFilesByPath.get(fileName);
    assert(entry, `manifest missing file entry for ${fileName}`);
    const filePath = path.join(packageDir, fileName);
    const stats = fs.statSync(filePath);
    assert(stats.isFile(), `${fileName} must be a file`);
    assert(entry.size_bytes === stats.size, `${fileName} size_bytes must match actual size`);
    assert(entry.sha256 === sha256File(filePath), `${fileName} sha256 must match actual file hash`);
  }

  const manifestHash = sha256File(resolvedManifestPath);
  const checksumsPath = path.join(packageDir, "SHA256SUMS.txt");
  const checksumLines = readTextFile(checksumsPath).trimEnd().split(/\r?\n/);
  const expectedChecksumEntries = [...expectedFileNames.map((name) => [name, manifestFilesByPath.get(name).sha256]), ["manifest.json", manifestHash]];
  expectedChecksumEntries.sort((left, right) => left[0].localeCompare(right[0]));
  const expectedChecksumLines = expectedChecksumEntries.map(([fileName, hash]) => `${hash}  ${fileName}`);
  assert(
    checksumLines.length === expectedChecksumLines.length &&
      checksumLines.every((line, index) => line === expectedChecksumLines[index]),
    "SHA256SUMS.txt must match the package files and manifest"
  );

  return {
    manifestPath: resolvedManifestPath,
    packageDir,
    manifest,
    manifestHash,
    fileEntries: expectedFileNames.map((fileName) => ({
      ...manifestFilesByPath.get(fileName),
      path: fileName
    }))
  };
}

export function expectedBinaryName(binName, target) {
  return target.includes("windows") ? `${binName}.exe` : binName;
}

export function runNodeScript(scriptPath, args = [], options = {}) {
  const result = spawnSync(process.execPath, [scriptPath, ...args], {
    cwd: options.cwd ?? root,
    env: options.env ?? process.env,
    stdio: options.stdio ?? "inherit",
    encoding: options.encoding ?? "utf8",
    windowsHide: true,
    maxBuffer: 1024 * 1024 * 64
  });

  if (result.error || result.status !== 0) {
    throw new Error(composeSpawnError(`node ${path.relative(root, scriptPath)}`, result));
  }

  return result;
}

export function buildReleasePackage(target = resolveHostTarget()) {
  const version = resolveWorkspaceVersion();
  const layout = resolvePackageLayout(version, target);
  fs.rmSync(layout.releaseRoot, { recursive: true, force: true });
  fs.mkdirSync(layout.packageDir, { recursive: true });
  try {
    const files = [];
    for (const component of releaseComponents) {
      const executablePath = buildReleaseBinary(component.packageName, component.binName, target);
      const destPath = resolveBinaryDestPath(layout.packageDir, executablePath);
      copyExecutable(executablePath, destPath);
      const fileStats = fs.statSync(destPath);
      files.push({
        path: normalizeRelativePath(path.basename(destPath)),
        component: component.component,
        ...(component.adapterId ? { adapter_id: component.adapterId } : {}),
        size_bytes: fileStats.size,
        sha256: sha256File(destPath)
      });
    }

    const manifest = {
      schema_version: 1,
      product: "docnav",
      version,
      target,
      generated_at: new Date().toISOString(),
      git_commit: getGitCommit(),
      source_dirty: isSourceDirty(),
      producer: resolveProducerMetadata(),
    files: files.sort((left, right) => compareStrings(left.path, right.path))
  };

    writeJsonFile(layout.manifestPath, manifest);

    const checksumEntries = [...manifest.files.map((entry) => [entry.path, entry.sha256]), ["manifest.json", sha256File(layout.manifestPath)]];
  checksumEntries.sort((left, right) => compareStrings(left[0], right[0]));
    writeTextFile(
      layout.checksumsPath,
      `${checksumEntries.map(([fileName, hash]) => `${hash}  ${fileName}`).join("\n")}\n`
    );

    validateReleasePackage(layout.manifestPath);

    return {
      ...layout,
      manifest,
      fileCount: manifest.files.length
    };
  } catch (error) {
    fs.rmSync(layout.releaseRoot, { recursive: true, force: true });
    throw error;
  }
}

function findCargoExecutable(output, binName) {
  let executable = null;
  for (const line of output.split(/\r?\n/)) {
    if (line.trim().length === 0) {
      continue;
    }
    let message;
    try {
      message = JSON.parse(line);
    } catch {
      continue;
    }
    if (
      message.reason === "compiler-artifact" &&
      message.executable &&
      message.target?.name === binName &&
      message.target?.kind?.includes("bin")
    ) {
      executable = message.executable;
    }
  }
  return executable;
}

function requiredEnv(name) {
  const value = process.env[name];
  if (!value) {
    throw new Error(`${name} is required`);
  }
  return value;
}

function requiredIntEnv(name) {
  const value = requiredEnv(name);
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(`${name} must be a positive integer`);
  }
  return parsed;
}

function compareStrings(left, right) {
  if (left < right) {
    return -1;
  }
  if (left > right) {
    return 1;
  }
  return 0;
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

function composeSpawnError(command, result) {
  const details = [];
  if (result.stdout) {
    details.push(`stdout:\n${result.stdout}`);
  }
  if (result.stderr) {
    details.push(`stderr:\n${result.stderr}`);
  }
  if (result.error) {
    details.push(`spawn error: ${result.error.message}`);
  }
  const exitText = result.status === null || result.status === undefined ? "spawn-error" : String(result.status);
  return `${command} failed with exit ${exitText}${details.length > 0 ? `\n${details.join("\n")}` : ""}`;
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}
