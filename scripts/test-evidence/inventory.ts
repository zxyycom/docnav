import fs from "node:fs";
import path from "node:path";

import {
  canonicalJson,
  sha256
} from "./fingerprint.ts";
import {
  diagnostic,
  type DiscoveryResult,
  type NativeTestEntry,
  type NativeTestInventory,
  type TestEvidenceDiagnostic
} from "./model.ts";

export function inventoryPath(workspaceRoot: string): string {
  return path.join(
    workspaceRoot,
    "docs",
    "test-evidence",
    "native-test-inventory.json"
  );
}

export function createNativeTestInventory(
  discovery: DiscoveryResult
): NativeTestInventory {
  const source = {
    profile: discovery.profile,
    entries: discovery.entries
  };
  return {
    schemaVersion: 1,
    profile: discovery.profile,
    sourceRevision: sha256(canonicalJson(source)),
    entries: discovery.entries
  };
}

export function compareNativeTestInventory(options: {
  expected: NativeTestInventory;
  actual: unknown;
  sourcePath: string;
}): TestEvidenceDiagnostic[] {
  const diagnostics: TestEvidenceDiagnostic[] = [];
  if (!isRecord(options.actual) || !Array.isArray(options.actual.entries)) {
    return [
      diagnostic(
        "inventory-invalid",
        "inventory",
        "committed native test inventory has an invalid shape",
        { path: options.sourcePath }
      )
    ];
  }
  const actualEntries = options.actual.entries
    .filter(isNativeTestEntry);
  if (actualEntries.length !== options.actual.entries.length) {
    diagnostics.push(diagnostic(
      "inventory-invalid",
      "inventory",
      "committed native test inventory contains an invalid Entry",
      { path: options.sourcePath }
    ));
  }

  const expectedMap = new Map(
    options.expected.entries.map((entry) => [entry.entryKey, entry])
  );
  const actualMap = new Map<string, NativeTestEntry>();
  for (const entry of actualEntries) {
    if (actualMap.has(entry.entryKey)) {
      diagnostics.push(diagnostic(
        "duplicate-case",
        "inventory",
        `inventory contains duplicate machine case ${entry.entryKey}`,
        {
          path: options.sourcePath,
          entryKey: entry.entryKey
        }
      ));
    }
    actualMap.set(entry.entryKey, entry);
  }

  for (const [entryKey, expected] of expectedMap) {
    const actual = actualMap.get(entryKey);
    if (!actual) {
      diagnostics.push(diagnostic(
        "missing-case",
        "inventory",
        `current native test entry has no machine case ${entryKey}`,
        {
          runner: expected.runner,
          target: expected.target,
          selector: expected.selector,
          entryKey,
          path: expected.sourcePath,
          line: expected.sourceRange.startLine,
          column: expected.sourceRange.startColumn
        }
      ));
    } else if (canonicalJson(actual) !== canonicalJson(expected)) {
      diagnostics.push(diagnostic(
        "stale-case",
        "inventory",
        `machine case ${entryKey} has stale source identity or fingerprint`,
        {
          runner: expected.runner,
          target: expected.target,
          selector: expected.selector,
          entryKey,
          path: expected.sourcePath
        }
      ));
    }
  }
  for (const [entryKey, actual] of actualMap) {
    if (!expectedMap.has(entryKey)) {
      diagnostics.push(diagnostic(
        "orphan-case",
        "inventory",
        `machine case has no current native test entry ${entryKey}`,
        {
          runner: actual.runner,
          target: actual.target,
          selector: actual.selector,
          entryKey,
          path: actual.sourcePath
        }
      ));
    }
  }

  if (
    !isRecord(options.actual.profile) ||
    canonicalJson(options.actual.profile) !== canonicalJson(options.expected.profile)
  ) {
    diagnostics.push(diagnostic(
      "inventory-profile-stale",
      "inventory",
      "machine inventory profile identity is stale",
      { path: options.sourcePath }
    ));
  }
  if (options.actual.sourceRevision !== options.expected.sourceRevision) {
    diagnostics.push(diagnostic(
      "inventory-revision-stale",
      "inventory",
      "machine inventory sourceRevision is stale",
      { path: options.sourcePath }
    ));
  }
  return diagnostics;
}

export function readCommittedInventory(
  workspaceRoot: string
): unknown {
  const sourcePath = inventoryPath(workspaceRoot);
  if (!fs.existsSync(sourcePath)) {
    return null;
  }
  try {
    return JSON.parse(fs.readFileSync(sourcePath, "utf8")) as unknown;
  } catch {
    return undefined;
  }
}

export function writeNativeTestInventory(
  workspaceRoot: string,
  inventory: NativeTestInventory
): void {
  const targetPath = inventoryPath(workspaceRoot);
  fs.mkdirSync(path.dirname(targetPath), { recursive: true });
  const temporaryPath = path.join(
    path.dirname(targetPath),
    `.${path.basename(targetPath)}.${process.pid}.tmp`
  );
  try {
    fs.writeFileSync(
      temporaryPath,
      `${JSON.stringify(inventory, null, 2)}\n`,
      { flag: "wx" }
    );
    fs.renameSync(temporaryPath, targetPath);
  } finally {
    fs.rmSync(temporaryPath, { force: true });
  }
}

function isNativeTestEntry(value: unknown): value is NativeTestEntry {
  return (
    isRecord(value) &&
    typeof value.entryKey === "string" &&
    typeof value.runner === "string" &&
    typeof value.target === "string" &&
    typeof value.selector === "string" &&
    typeof value.sourcePath === "string" &&
    isRecord(value.sourceRange) &&
    typeof value.sourceFingerprint === "string"
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
