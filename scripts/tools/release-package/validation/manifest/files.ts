import { compareStrings } from "../../config.ts";
import type { ReleaseManifestFile } from "../../config.ts";
import { isRecord } from "../../../foundation/src/type-guards.ts";
import {
  assert,
  assertEqualLists,
  assertNonEmptyString,
} from "../assertions.ts";

export function validateManifestFiles(
  files: ReleaseManifestFile[],
  expectedFileNames: string[],
): Map<string, ReleaseManifestFile> {
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

export function validateManifestFile(
  entry: unknown,
): asserts entry is ReleaseManifestFile {
  assert(isRecord(entry), "manifest file entry must be an object");
  assertNonEmptyString(entry.path, "manifest file path");
  assert(
    !entry.path.includes("/") && !entry.path.includes("\\"),
    "manifest file path must be a file name",
  );
  assert(entry.component === "core", "manifest file component must be core");
  const sizeBytes = entry.size_bytes;
  assert(
    typeof sizeBytes === "number" &&
      Number.isInteger(sizeBytes) &&
      sizeBytes >= 0,
    "manifest file size_bytes must be an integer",
  );
  assert(
    typeof entry.sha256 === "string" && /^[0-9a-f]{64}$/.test(entry.sha256),
    "manifest file sha256 must be lowercase hex",
  );

  assert(
    entry.adapter_id === undefined,
    "core manifest entry must not include adapter_id",
  );
}
