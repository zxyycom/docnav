import fs from "node:fs";
import path from "node:path";

import {
  compareStrings,
  expectedBinaryName,
  releaseComponents,
} from "./config.ts";
import type { ReleaseManifest, ReleaseManifestFile, ReleaseProducer } from "./config.ts";
import { readJsonFile, readTextFile } from "../fs.ts";
import { isRecord } from "../type-guards.ts";
import { sha256File } from "./io.ts";

export type ManifestValidationOptions = {
  expectProducerKind?: ReleaseProducer["kind"] | null;
  expectSourceDirty?: boolean | null;
};

export function validateReleasePackage(manifestPath: string, options: ManifestValidationOptions = {}) {
  const resolvedManifestPath = path.resolve(manifestPath);
  const packageDir = path.dirname(resolvedManifestPath);
  const manifest = readJsonFile(resolvedManifestPath);

  validateManifestMetadata(manifest, options);
  validateManifestLocation(resolvedManifestPath, packageDir, manifest);

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

function validateManifestLocation(manifestPath: string, packageDir: string, manifest: ReleaseManifest): void {
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

function validateManifestMetadata(manifest: unknown, options: ManifestValidationOptions): asserts manifest is ReleaseManifest {
  assert(isRecord(manifest), "manifest root must be an object");
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

  const producer = manifest.producer;
  validateProducer(producer);
  for (const entry of manifest.files) {
    validateManifestFile(entry);
  }

  if (options.expectProducerKind) {
    assert(
      producer.kind === options.expectProducerKind,
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

function validateProducer(producer: unknown): asserts producer is ReleaseProducer {
  assert(isRecord(producer), "manifest.producer must be an object");
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

function validateManifestFiles(files: ReleaseManifestFile[], expectedFileNames: string[]): Map<string, ReleaseManifestFile> {
  const actualFileNames = files.map((entry) => entry.path);
  const expectedSortedFiles = [...expectedFileNames].sort(compareStrings);
  const actualSortedFiles = [...actualFileNames].sort(compareStrings);
  assertEqualLists(
    actualSortedFiles,
    expectedSortedFiles,
    `manifest.files must list exactly ${expectedSortedFiles.join(", ")}`,
  );

  const filesByPath = new Map<string, ReleaseManifestFile>();
  for (const entry of files) {
    validateManifestFile(entry);
    filesByPath.set(entry.path, entry);
  }
  return filesByPath;
}

function validateManifestFile(entry: unknown): asserts entry is ReleaseManifestFile {
  assert(isRecord(entry), "manifest file entry must be an object");
  assertNonEmptyString(entry.path, "manifest file path");
  assert(
    !entry.path.includes("/") && !entry.path.includes("\\"),
    "manifest file path must be a file name",
  );
  assert(
    entry.component === "core" || entry.component === "adapter",
    "manifest file component must be core or adapter",
  );
  const sizeBytes = entry.size_bytes;
  assert(typeof sizeBytes === "number" && Number.isInteger(sizeBytes) && sizeBytes >= 0, "manifest file size_bytes must be an integer");
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

function validatePackageDirectory(packageDir: string, expectedFileNames: string[]): void {
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
  packageDir: string,
  expectedFileNames: string[],
  manifestFilesByPath: Map<string, ReleaseManifestFile>,
): void {
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
  packageDir: string,
  manifestPath: string,
  expectedFileNames: string[],
  manifestFilesByPath: Map<string, ReleaseManifestFile>,
  manifestHash: string,
): void {
  const checksumLines = readTextFile(path.join(packageDir, "SHA256SUMS.txt"))
    .trimEnd()
    .split(/\r?\n/);
  const checksumEntries = [
    ...expectedFileNames.map((name) => [
      name,
      requiredManifestEntry(manifestFilesByPath, name).sha256,
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

function assertEqualLists(actual: string[], expected: string[], message: string): void {
  assert(
    actual.length === expected.length &&
      actual.every((value, index) => value === expected[index]),
    message,
  );
}

function assertNonEmptyString(value: unknown, label: string): asserts value is string {
  assert(
    typeof value === "string" && value.length > 0,
    `${label} must be a string`,
  );
}

function assertPositiveInteger(value: unknown, label: string): asserts value is number {
  assert(
    typeof value === "number" && Number.isInteger(value) && value > 0,
    `${label} must be a positive integer`,
  );
}

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

function requiredManifestEntry(
  manifestFilesByPath: Map<string, ReleaseManifestFile>,
  fileName: string,
): ReleaseManifestFile {
  const entry = manifestFilesByPath.get(fileName);
  assert(entry, `manifest missing file entry for ${fileName}`);
  return entry;
}
