import fs from "node:fs";
import path from "node:path";

import { compareStrings } from "../config.ts";
import type { ReleaseManifestFile } from "../config.ts";
import { readTextFile } from "../../fs.ts";
import { sha256File } from "../io.ts";
import { assert, assertEqualLists } from "./assertions.ts";

export function validatePackageDirectory(
  packageDir: string,
  expectedFileNames: string[],
): void {
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

export function validatePackagedBinaries(
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

export function validateChecksums(
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

function requiredManifestEntry(
  manifestFilesByPath: Map<string, ReleaseManifestFile>,
  fileName: string,
): ReleaseManifestFile {
  const entry = manifestFilesByPath.get(fileName);
  assert(entry, `manifest missing file entry for ${fileName}`);
  return entry;
}
