import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { closeStaticAndRuntimeEntries } from "../test-evidence/closure.ts";
import {
  compareNativeTestInventory,
  createNativeTestInventory
} from "../test-evidence/inventory.ts";
import type {
  DiscoveryResult,
  NativeTestEntry,
  StaticTestCandidate
} from "../test-evidence/model.ts";

describe("native test evidence closure", () => {
  it("creates one machine case from one static and runtime entry", () => {
    const result = closeStaticAndRuntimeEntries({
      runner: "bun",
      statics: [staticCandidate("fixture\u00001\u0000case")],
      runtime: [
        {
          identity: "fixture\u00001\u0000case",
          target: "fixture.test.ts",
          selector: "suite > case"
        }
      ],
      createEntryKey: ({ target, selector }) => `bun|${target}|${selector}`
    });

    assert.deepEqual(result.diagnostics, []);
    assert.deepEqual(
      result.entries.map(({ entryKey }) => entryKey),
      ["bun|fixture.test.ts|suite > case"]
    );
  });

  it("reports a static-only declaration", () => {
    const result = closeStaticAndRuntimeEntries({
      runner: "bun",
      statics: [staticCandidate("fixture\u00001\u0000case")],
      runtime: [],
      createEntryKey: ({ target, selector }) => `bun|${target}|${selector}`
    });

    assertDiagnostic(result.diagnostics, "static-only");
  });

  it("reports a runtime-only entry", () => {
    const result = closeStaticAndRuntimeEntries({
      runner: "cargo",
      statics: [],
      runtime: [
        {
          identity: "case",
          target: "package:lib:target",
          selector: "tests::case"
        }
      ],
      createEntryKey: ({ target, selector }) => `cargo|${target}|${selector}`
    });

    assertDiagnostic(result.diagnostics, "runtime-only");
  });

  it("rejects ambiguous duplicate identities instead of guessing", () => {
    const result = closeStaticAndRuntimeEntries({
      runner: "smoke",
      statics: [
        staticCandidate("CORE-DUPLICATE-001"),
        staticCandidate("CORE-DUPLICATE-001")
      ],
      runtime: [
        {
          identity: "CORE-DUPLICATE-001",
          target: "core:root",
          selector: "CORE-DUPLICATE-001"
        }
      ],
      createEntryKey: ({ target, selector }) => `smoke|${target}|${selector}`
    });

    assertDiagnostic(result.diagnostics, "duplicate-entry");
    assert.deepEqual(result.entries, []);
  });

  it("distinguishes missing, orphan and stale machine cases", () => {
    const expectedEntry = nativeEntry("bun|fixture.test.ts|suite > case");
    const expected = createNativeTestInventory(discovery([expectedEntry]));
    const actualEntry = {
      ...nativeEntry("bun|fixture.test.ts|orphan"),
      sourceFingerprint: `sha256:${"2".repeat(64)}`
    };
    const actual = {
      ...expected,
      sourceRevision: `sha256:${"3".repeat(64)}`,
      entries: [
        {
          ...expectedEntry,
          sourceFingerprint: `sha256:${"4".repeat(64)}`
        },
        actualEntry
      ]
    };

    const diagnostics = compareNativeTestInventory({
      expected,
      actual,
      sourcePath: "docs/test-evidence/native-test-inventory.json"
    });

    assertDiagnostic(diagnostics, "stale-case");
    assertDiagnostic(diagnostics, "orphan-case");
    assertDiagnostic(diagnostics, "inventory-revision-stale");

    const missing = compareNativeTestInventory({
      expected,
      actual: {
        ...expected,
        entries: []
      },
      sourcePath: "docs/test-evidence/native-test-inventory.json"
    });
    assertDiagnostic(missing, "missing-case");
  });
});

function staticCandidate(identity: string): StaticTestCandidate {
  return {
    identity,
    sourcePath: "fixture.test.ts",
    sourceRange: {
      startLine: 1,
      startColumn: 1,
      endLine: 1,
      endColumn: 10
    },
    sourceFingerprint: `sha256:${"0".repeat(64)}`
  };
}

function nativeEntry(entryKey: string): NativeTestEntry {
  return {
    entryKey,
    runner: "bun",
    target: "fixture.test.ts",
    selector: entryKey.split("|").at(-1) ?? entryKey,
    sourcePath: "fixture.test.ts",
    sourceRange: {
      startLine: 1,
      startColumn: 1,
      endLine: 1,
      endColumn: 10
    },
    sourceFingerprint: `sha256:${"1".repeat(64)}`
  };
}

function discovery(entries: NativeTestEntry[]): DiscoveryResult {
  return {
    profile: {
      id: "fixture",
      version: 1
    },
    entries,
    diagnostics: []
  };
}

function assertDiagnostic(
  diagnostics: Array<{ code: string }>,
  code: string
): void {
  assert.ok(
    diagnostics.some((diagnostic) => diagnostic.code === code),
    `expected diagnostic ${code}: ${JSON.stringify(diagnostics)}`
  );
}
