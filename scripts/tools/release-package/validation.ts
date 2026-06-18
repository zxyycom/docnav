import fs from "node:fs";
import path from "node:path";

import {
  compareStrings,
  expectedBinaryName,
  releaseComponents,
} from "./config.ts";
import { readJsonFile, readTextFile, sha256File } from "./io.ts";

export function resolveReleaseManifest(manifestPath: ExternalValue) {
  const packageDir = path.dirname(path.resolve(manifestPath));
  return {
    packageDir,
    manifest: readJsonFile(manifestPath),
  };
}

export function validateReleasePackage(manifestPath: ExternalValue, options: ExternalValue = {}) {
  const resolvedManifestPath = path.resolve(manifestPath);
  const packageDir = path.dirname(resolvedManifestPath);
  const manifest = readJsonFile(resolvedManifestPath);

  validateManifestLocation(resolvedManifestPath, packageDir, manifest);
  validateManifestMetadata(manifest, options);

  const expectedFileNames = releaseComponents.map((component) =>
    expectedBinaryName(component.binName, manifest.target),
  );
  const manifestFilesByPath = validateManifestFiles(
    manifest.files,
    expectedFileNames,
  );

  validatePackageDirectory(packageDir, expectedFileNames);
  validatePackagedBinaries(packageDir, expectedFileNames, manifestFilesByPath);

  const manifestHash = sha256File(resolvedManifestPath);
  validateChecksums(
    packageDir,
    resolvedManifestPath,
    expectedFileNames,
    manifestFilesByPath,
    manifestHash,
  );

  return {
    manifestPath: resolvedManifestPath,
    packageDir,
    manifest,
    manifestHash,
    fileEntries: expectedFileNames.map((fileName) => ({
      ...manifestFilesByPath.get(fileName),
      path: fileName,
    })),
  };
}

function validateManifestLocation(manifestPath: ExternalValue, packageDir: ExternalValue, manifest: ExternalValue) {
  assert(
    path.basename(manifestPath) === "manifest.json",
    "manifest path must end with manifest.json",
  );
  assert(
    path.basename(packageDir) === "package",
    "manifest must live in a package/ directory",
  );
  assert(
    path.basename(path.dirname(packageDir)) === manifest.target,
    "package target directory must match manifest target",
  );
  assert(
    path.basename(path.dirname(path.dirname(packageDir))) ===
      `v${manifest.version}`,
    "package version directory must match manifest version",
  );
  assert(
    path.basename(path.dirname(path.dirname(path.dirname(packageDir)))) ===
      "docnav",
    "package root must be artifacts/docnav",
  );
}

function validateManifestMetadata(manifest: ExternalValue, options: ExternalValue) {
  assert(manifest.schema_version === 1, "manifest.schema_version must be 1");
  assert(manifest.product === "docnav", "manifest.product must be docnav");
  assertNonEmptyString(manifest.version, "manifest.version");
  assertNonEmptyString(manifest.target, "manifest.target");
  assertNonEmptyString(manifest.generated_at, "manifest.generated_at");
  assertNonEmptyString(manifest.git_commit, "manifest.git_commit");
  assert(
    typeof manifest.source_dirty === "boolean",
    "manifest.source_dirty must be a boolean",
  );
  assert(Array.isArray(manifest.files), "manifest.files must be an array");
  assert(
    manifest.files.length === releaseComponents.length,
    "manifest.files must list all release binaries",
  );

  validateProducer(manifest.producer ?? {});

  if (options.expectProducerKind) {
    assert(
      manifest.producer.kind === options.expectProducerKind,
      `manifest.producer.kind must be ${options.expectProducerKind}`,
    );
  }
  if (
    options.expectSourceDirty !== null &&
    options.expectSourceDirty !== undefined
  ) {
    assert(
      manifest.source_dirty === options.expectSourceDirty,
      `manifest.source_dirty must be ${options.expectSourceDirty}`,
    );
  }
}

function validateProducer(producer: ExternalValue) {
  assert(
    producer.kind === "local" || producer.kind === "github-actions",
    "manifest.producer.kind must be local or github-actions",
  );

  if (producer.kind === "local") {
    assert(producer.workflow === null, "local producer.workflow must be null");
    assert(producer.run_id === null, "local producer.run_id must be null");
    assert(
      producer.run_attempt === null,
      "local producer.run_attempt must be null",
    );
    return;
  }

  assert(
    typeof producer.workflow === "string" && producer.workflow.length > 0,
    "github-actions producer.workflow must be present",
  );
  assertPositiveInteger(producer.run_id, "github-actions producer.run_id");
  assertPositiveInteger(
    producer.run_attempt,
    "github-actions producer.run_attempt",
  );
}

function validateManifestFiles(files: ExternalValue, expectedFileNames: ExternalValue) {
  const actualFileNames = files.map((entry: ExternalValue) => entry.path);
  const expectedSortedFiles = [...expectedFileNames].sort(compareStrings);
  const actualSortedFiles = [...actualFileNames].sort(compareStrings);
  assertEqualLists(
    actualSortedFiles,
    expectedSortedFiles,
    `manifest.files must list exactly ${expectedSortedFiles.join(", ")}`,
  );

  const filesByPath = new Map();
  for (const entry of files) {
    validateManifestFile(entry);
    filesByPath.set(entry.path, entry);
  }
  return filesByPath;
}

function validateManifestFile(entry: ExternalValue) {
  assertNonEmptyString(entry.path, "manifest file path");
  assert(
    !entry.path.includes("/") && !entry.path.includes("\\"),
    "manifest file path must be a file name",
  );
  assert(
    entry.component === "core" || entry.component === "adapter",
    "manifest file component must be core or adapter",
  );
  assert(
    Number.isInteger(entry.size_bytes) && entry.size_bytes >= 0,
    "manifest file size_bytes must be an integer",
  );
  assert(
    typeof entry.sha256 === "string" && /^[0-9a-f]{64}$/.test(entry.sha256),
    "manifest file sha256 must be lowercase hex",
  );

  if (entry.component === "adapter") {
    assert(
      entry.adapter_id === "docnav-markdown",
      "adapter manifest entry must include adapter_id",
    );
  } else {
    assert(
      entry.adapter_id === undefined,
      "core manifest entry must not include adapter_id",
    );
  }
}

function validatePackageDirectory(packageDir: ExternalValue, expectedFileNames: ExternalValue) {
  const actualEntries = fs.readdirSync(packageDir).sort(compareStrings);
  const expectedEntries = [
    ...expectedFileNames,
    "manifest.json",
    "SHA256SUMS.txt",
  ].sort(compareStrings);
  assertEqualLists(
    actualEntries,
    expectedEntries,
    `package directory must contain exactly ${expectedEntries.join(", ")}`,
  );
}

function validatePackagedBinaries(
  packageDir: ExternalValue,
  expectedFileNames: ExternalValue,
  manifestFilesByPath: ExternalValue,
) {
  for (const fileName of expectedFileNames) {
    const entry = manifestFilesByPath.get(fileName);
    assert(entry, `manifest missing file entry for ${fileName}`);

    const filePath = path.join(packageDir, fileName);
    const stats = fs.statSync(filePath);
    assert(stats.isFile(), `${fileName} must be a file`);
    assert(
      entry.size_bytes === stats.size,
      `${fileName} size_bytes must match actual size`,
    );
    assert(
      entry.sha256 === sha256File(filePath),
      `${fileName} sha256 must match actual file hash`,
    );
  }
}

function validateChecksums(
  packageDir: ExternalValue,
  manifestPath: ExternalValue,
  expectedFileNames: ExternalValue,
  manifestFilesByPath: ExternalValue,
  manifestHash: ExternalValue,
) {
  const checksumLines = readTextFile(path.join(packageDir, "SHA256SUMS.txt"))
    .trimEnd()
    .split(/\r?\n/);
  const checksumEntries = [
    ...expectedFileNames.map((name: ExternalValue) => [
      name,
      manifestFilesByPath.get(name).sha256,
    ]),
    [path.basename(manifestPath), manifestHash],
  ];
  checksumEntries.sort((left, right) => compareStrings(left[0], right[0]));
  const expectedLines = checksumEntries.map(
    ([fileName, hash]) => `${hash}  ${fileName}`,
  );

  assertEqualLists(
    checksumLines,
    expectedLines,
    "SHA256SUMS.txt must match the package files and manifest",
  );
}

function assertEqualLists(actual: ExternalValue, expected: ExternalValue, message: ExternalValue) {
  assert(
    actual.length === expected.length &&
      actual.every((value: ExternalValue, index: ExternalValue) => value === expected[index]),
    message,
  );
}

function assertNonEmptyString(value: ExternalValue, label: ExternalValue) {
  assert(
    typeof value === "string" && value.length > 0,
    `${label} must be a string`,
  );
}

function assertPositiveInteger(value: ExternalValue, label: ExternalValue) {
  assert(
    Number.isInteger(value) && value > 0,
    `${label} must be a positive integer`,
  );
}

function assert(condition: ExternalValue, message: ExternalValue) {
  if (!condition) {
    throw new Error(message);
  }
}
