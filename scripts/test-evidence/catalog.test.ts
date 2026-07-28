import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { Ajv2020 } from "ajv/dist/2020.js";
import type { AnySchema } from "ajv";

import {
  queryTestEvidence,
  showTestEvidence,
  syncTestEvidenceIndex,
  validateTestEvidence
} from "../../.codex/skills/test-evidence-review/scripts/test-evidence-catalog.mjs";
import { compareInventoryBaseline } from "./change-report.ts";
import { exitCodeForDiagnostics } from "./cli.ts";
import { discoverNativeTestEntries } from "./discover.ts";
import { parseBunJUnit } from "./discovery/bun.ts";
import { resolveBunTestFiles } from "./discovery/bun-files.ts";
import { parseLibtestList } from "./discovery/rust.ts";
import { createNativeTestInventory } from "./inventory.ts";
import { parseNativeTestInventory } from "./inventory-validation.ts";
import {
  diagnostic,
  type DiscoveryResult,
  type NativeTestEntry
} from "./model.ts";
import {
  loadSupportedRunnerProfile,
  profilePath,
  workspaceRoot
} from "./profile.ts";

const entryKey = "bun|tests/example.test.ts|contract > rejects invalid input";

test("validates, indexes and queries NativeTestEntry and Evidence Claim", () => {
  using fixture = createEvidenceFixture();
  writeClaim(fixture.root);

  const sync = syncTestEvidenceIndex({
    mode: "write",
    workspaceRoot: fixture.root
  });
  const report = validateTestEvidence({ workspaceRoot: fixture.root });
  const byOwner = queryTestEvidence({
    workspaceRoot: fixture.root,
    ownerRef: "docs/owner.md#contract"
  });
  const shown = showTestEvidence({
    workspaceRoot: fixture.root,
    id: entryKey
  });

  assert.equal(sync.status, "ok");
  assert.equal(report.status, "ok");
  assert.deepEqual(report.summary, {
    topics: 1,
    entries: 1,
    claims: 1
  });
  assert.equal(byOwner.source, "index");
  assert.deepEqual(
    byOwner.items.map(({ kind, id }) => ({ kind, id })),
    [
      { kind: "claim", id: "CLAIM-EXAMPLE-CONTRACT-001" },
      { kind: "entry", id: entryKey }
    ]
  );
  assert.equal(shown.item?.kind, "entry");
  assert.deepEqual(
    shown.item?.kind === "entry" ? shown.item.claimIds : [],
    ["CLAIM-EXAMPLE-CONTRACT-001"]
  );
});

test("allows a current machine case without an Evidence Claim", () => {
  using fixture = createEvidenceFixture();
  const sync = syncTestEvidenceIndex({
    mode: "write",
    workspaceRoot: fixture.root
  });
  const report = validateTestEvidence({ workspaceRoot: fixture.root });

  assert.equal(sync.status, "ok");
  assert.equal(report.status, "ok");
  assert.equal(report.summary.entries, 1);
  assert.equal(report.summary.claims, 0);
});

test("rejects unknown Entry support and no-information templates", () => {
  using fixture = createEvidenceFixture();
  writeClaim(fixture.root, {
    statement: "Stable contract.",
    supportedBy: "bun|tests/missing.test.ts|missing"
  });

  const report = validateTestEvidence({ workspaceRoot: fixture.root });
  assertDiagnostic(report.diagnostics, "claim.template-repetition");
  assertDiagnostic(report.diagnostics, "claim.entry-unknown");
});

test("reports claim-stale when owner content changes after index sync", () => {
  using fixture = createEvidenceFixture();
  writeClaim(fixture.root);
  const sync = syncTestEvidenceIndex({
    mode: "write",
    workspaceRoot: fixture.root
  });
  assert.equal(sync.status, "ok");

  fs.writeFileSync(
    path.join(fixture.root, "docs", "owner.md"),
    "# Owner\n\n## Contract\n\nChanged owner requirement.\n"
  );
  const report = validateTestEvidence({ workspaceRoot: fixture.root });

  assertDiagnostic(report.diagnostics, "claim.stale");
  assertDiagnostic(report.diagnostics, "index.stale");
});

test("uses a warning-only memory projection without writing a missing index", () => {
  using fixture = createEvidenceFixture();
  const indexPath = path.join(
    fixture.root,
    "docs",
    "test-evidence",
    "test-evidence-index.json"
  );

  const result = queryTestEvidence({
    workspaceRoot: fixture.root,
    kind: "entry"
  });

  assert.equal(result.status, "ok");
  assert.equal(result.source, "memory");
  assertDiagnostic(result.diagnostics, "index.missing", false);
  assert.equal(fs.existsSync(indexPath), false);
  assert.equal(result.total, 1);
});

test("validates the committed profile, inventory, Claim and index examples against their schemas", () => {
  const ajv = new Ajv2020({
    allErrors: true,
    strict: true
  });
  const schemaRoot = path.join(
    workspaceRoot,
    ".codex",
    "skills",
    "test-evidence-review",
    "schemas"
  );
  for (const fileName of [
    "native-test-entry.schema.json",
    "native-test-inventory.schema.json",
    "evidence-claim.schema.json",
    "claim-topic-catalog.schema.json",
    "test-evidence-index.schema.json"
  ]) {
    ajv.addSchema(readJson(path.join(schemaRoot, fileName)) as AnySchema);
  }
  ajv.addSchema(readJson(path.join(
    workspaceRoot,
    "scripts",
    "test-evidence",
    "supported-runner-profile.schema.json"
  )) as AnySchema);

  const committedIndex = readJson(path.join(
    workspaceRoot,
    "docs",
    "test-evidence",
    "test-evidence-index.json"
  ));
  assert.ok(
    typeof committedIndex === "object" &&
    committedIndex !== null &&
    !Array.isArray(committedIndex)
  );
  const committedClaims = (
    committedIndex as Record<string, unknown>
  ).claims;
  assert.ok(Array.isArray(committedClaims));
  const examples: Array<[string, unknown]> = [
    [
      "https://docnav.dev/test-evidence/supported-runner-profile.schema.json",
      readJson(profilePath)
    ],
    [
      "https://docnav.dev/test-evidence/native-test-inventory.schema.json",
      readJson(path.join(
        workspaceRoot,
        "docs",
        "test-evidence",
        "native-test-inventory.json"
      ))
    ],
    [
      "https://docnav.dev/test-evidence/claim-topic-catalog.schema.json",
      readJson(path.join(
        workspaceRoot,
        "docs",
        "test-evidence",
        "claim-topics.json"
      ))
    ],
    [
      "https://docnav.dev/test-evidence/test-evidence-index.schema.json",
      committedIndex
    ]
  ];
  for (const [schemaId, value] of examples) {
    assert.equal(
      ajv.validate(schemaId, value),
      true,
      `${schemaId}: ${ajv.errorsText()}`
    );
  }
  for (const claim of committedClaims) {
    assert.equal(
      ajv.validate(
        "https://docnav.dev/test-evidence/evidence-claim.schema.json",
        claim
      ),
      true,
      `evidence Claim: ${ajv.errorsText()}`
    );
  }
});

test("parses stable Cargo and Bun runner reports without inferring missing fields", () => {
  assert.deepEqual(
    parseLibtestList([
      "tests::first: test",
      "tests::ignored_but_selectable: test",
      "benchmark: benchmark",
      ""
    ].join("\n")),
    [
      "tests::first",
      "tests::ignored_but_selectable"
    ]
  );
  assert.deepEqual(
    parseBunJUnit([
      "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
      "<testsuites tests=\"1\" failures=\"0\">",
      "  <testcase name=\"rejects &quot;bad&quot; input\" classname=\"suite\" file=\"tests/example.test.ts\" line=\"7\" />",
      "</testsuites>"
    ].join("\n")),
    [
      {
        name: "rejects \"bad\" input",
        className: "suite",
        file: "tests/example.test.ts",
        line: 7
      }
    ]
  );
  assert.throws(
    () => parseBunJUnit("<testsuites tests=\"1\" failures=\"0\"></testsuites>"),
    /contains 0 testcase/
  );
});

test("reports structural changes without replacing full-tree closure", () => {
  const original = exampleEntry("bun|tests/example.test.ts|suite > old", 10);
  const renamed = {
    ...exampleEntry("bun|tests/example.test.ts|suite > new", 11),
    sourceFingerprint: `sha256:${"2".repeat(64)}`
  };
  const unchanged = exampleEntry("cargo|package:lib:target|tests::same", 20);
  const changed = {
    ...unchanged,
    sourceFingerprint: `sha256:${"3".repeat(64)}`
  };
  const baseline = createNativeTestInventory(discovery([original, unchanged]));
  const current = createNativeTestInventory(discovery([renamed, changed]));

  const report = compareInventoryBaseline(baseline, current);

  assert.deepEqual(report.added, [renamed.entryKey]);
  assert.deepEqual(report.removed, [original.entryKey]);
  assert.deepEqual(report.implementationChanged, [changed.entryKey]);
  assert.deepEqual(report.renameCandidates, [
    {
      from: original.entryKey,
      to: renamed.entryKey
    }
  ]);
  assert.throws(
    () => parseNativeTestInventory({
      ...baseline,
      entries: [null]
    }),
    /entries\[0\]/
  );
});

test("uses distinct project exit statuses for discovery, runner, inventory and Claim failures", () => {
  assert.equal(exitCodeForDiagnostics([
    diagnostic("unsupported-entry-shape", "static", "unsupported")
  ]), 3);
  assert.equal(exitCodeForDiagnostics([
    diagnostic("runner-report-failed", "runner", "failed")
  ]), 4);
  assert.equal(exitCodeForDiagnostics([
    diagnostic("missing-case", "inventory", "missing")
  ]), 5);
  assert.equal(exitCodeForDiagnostics([
    diagnostic("claim.owner-unknown", "claim", "unknown")
  ]), 6);
});

test("loads one versioned and sorted supported runner profile", async () => {
  const profile = loadSupportedRunnerProfile();
  assert.equal(profile.schemaVersion, 2);
  assert.equal(profile.id, "docnav-native-tests");
  assert.equal(profile.version, 2);
  assert.deepEqual(profile.bun, {
    sourceRoots: ["scripts", "test"],
    include: ["**/*.test.ts"],
    ignore: [],
    supplementalFiles: []
  });
  assert.deepEqual(
    resolveBunTestFiles({ workspaceRoot, profile: profile.bun }),
    findConventionalBunTests(workspaceRoot, profile.bun.sourceRoots)
  );
  assert.equal(profile.smoke.factory, "test/smoke/core/profile.ts");

  const temporaryRoot = fs.mkdtempSync(path.join(
    os.tmpdir(),
    "docnav-runner-profile-"
  ));
  try {
    const invalidProfiles = [
      {
        ...profile,
        id: 1
      },
      {
        ...profile,
        cargo: {
          ...profile.cargo,
          sourceRoots: ["../outside"]
        }
      },
      {
        ...profile,
        bun: {
          ...profile.bun,
          sourceRoots: []
        }
      },
      {
        ...profile,
        bun: {
          ...profile.bun,
          include: ["../**/*.test.ts"]
        }
      },
      {
        ...profile,
        smoke: {
          ...profile.smoke,
          factory: "test/smoke/other/profile.ts"
        }
      },
      {
        ...profile,
        smoke: {
          ...profile.smoke,
          sourceRoots: ["/tmp"]
        }
      }
    ];
    for (const [index, invalidProfile] of invalidProfiles.entries()) {
      const sourcePath = path.join(temporaryRoot, `${index}.json`);
      writeJson(sourcePath, invalidProfile);
      assert.throws(
        () => loadSupportedRunnerProfile(sourcePath),
        /identity|safe relative POSIX paths|non-empty string array|positive relative POSIX globs|smoke identity/
      );
    }
  } finally {
    fs.rmSync(temporaryRoot, { force: true, recursive: true });
  }

  const rootMismatch = await discoverNativeTestEntries({
    workspaceRoot: os.tmpdir()
  });
  assert.ok(rootMismatch.diagnostics.some(({ code, message }) => (
    code === "runner-profile-invalid" &&
    message.includes("current checkout")
  )));
});

function createEvidenceFixture(): Fixture {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "docnav-test-evidence-"));
  fs.mkdirSync(path.join(root, "docs", "test-evidence"), { recursive: true });
  fs.writeFileSync(
    path.join(root, "docs", "owner.md"),
    "# Owner\n\n## Contract\n\nInvalid input is rejected without changing state.\n"
  );
  writeJson(
    path.join(root, "docs", "test-evidence", "claim-topics.json"),
    {
      schemaVersion: 1,
      topics: [
        {
          id: "contract",
          description: "Stable public contract evidence."
        }
      ]
    }
  );
  writeJson(
    path.join(root, "docs", "test-evidence", "native-test-inventory.json"),
    {
      schemaVersion: 1,
      profile: {
        id: "fixture",
        version: 1
      },
      sourceRevision: `sha256:${"0".repeat(64)}`,
      entries: [
        {
          entryKey,
          runner: "bun",
          target: "tests/example.test.ts",
          selector: "contract > rejects invalid input",
          sourcePath: "tests/example.test.ts",
          sourceRange: {
            startLine: 3,
            startColumn: 1,
            endLine: 5,
            endColumn: 3
          },
          sourceFingerprint: `sha256:${"1".repeat(64)}`
        }
      ]
    }
  );
  return {
    root,
    [Symbol.dispose]() {
      fs.rmSync(root, { force: true, recursive: true });
    }
  };
}

function findConventionalBunTests(
  root: string,
  sourceRoots: readonly string[]
): string[] {
  const files: string[] = [];
  for (const sourceRoot of sourceRoots) {
    visit(path.join(root, sourceRoot), sourceRoot);
  }
  return files.sort();

  function visit(directoryPath: string, relativeDirectory: string): void {
    for (const entry of fs.readdirSync(directoryPath, { withFileTypes: true })) {
      const relativePath = path.posix.join(relativeDirectory, entry.name);
      if (entry.isDirectory()) {
        visit(path.join(directoryPath, entry.name), relativePath);
      } else if (entry.isFile() && entry.name.endsWith(".test.ts")) {
        files.push(relativePath);
      }
    }
  }
}

function writeClaim(
  root: string,
  overrides: {
    statement?: string;
    supportedBy?: string;
  } = {}
): void {
  const claimRoot = path.join(
    root,
    "docs",
    "test-evidence",
    "claims",
    "contract"
  );
  fs.mkdirSync(claimRoot, { recursive: true });
  fs.writeFileSync(
    path.join(claimRoot, "example-contract.md"),
    [
      "# Claim CLAIM-EXAMPLE-CONTRACT-001: Invalid input remains rejected",
      "",
      "Topic: `contract`",
      "Owner ref: `docs/owner.md#contract`",
      "",
      "Statement:",
      `- ${overrides.statement ?? "Invalid input cannot mutate the protected state."}`,
      "",
      "Observations:",
      "- The call returns the documented invalid-input error.",
      "- The protected state remains unchanged.",
      "",
      "Supported by:",
      `- \`${overrides.supportedBy ?? entryKey}\``,
      ""
    ].join("\n")
  );
}

function writeJson(targetPath: string, value: unknown): void {
  fs.writeFileSync(targetPath, `${JSON.stringify(value, null, 2)}\n`);
}

function readJson(sourcePath: string): unknown {
  return JSON.parse(fs.readFileSync(sourcePath, "utf8")) as unknown;
}

function exampleEntry(entryKey: string, startLine: number): NativeTestEntry {
  const [runner = "bun", target = "tests/example.test.ts", selector = "case"] = entryKey.split("|");
  return {
    entryKey,
    runner,
    target,
    selector,
    sourcePath: "tests/example.test.ts",
    sourceRange: {
      startLine,
      startColumn: 1,
      endLine: startLine,
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
    entries: [...entries].sort((left, right) => (
      left.entryKey < right.entryKey ? -1 : left.entryKey > right.entryKey ? 1 : 0
    )),
    diagnostics: []
  };
}

function assertDiagnostic(
  diagnostics: Array<{ code: string; blocking: boolean }>,
  code: string,
  blocking = true
): void {
  assert.ok(
    diagnostics.some((diagnostic) => (
      diagnostic.code === code && diagnostic.blocking === blocking
    )),
    `expected ${blocking ? "blocking" : "non-blocking"} diagnostic ${code}: ${JSON.stringify(diagnostics)}`
  );
}

type Fixture = {
  root: string;
  [Symbol.dispose](): void;
};
