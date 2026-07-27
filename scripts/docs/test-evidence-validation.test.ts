import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, it } from "node:test";

import {
  syncTestEvidenceIndex,
  validateTestEvidence
} from "../../.codex/skills/test-evidence-review/scripts/test-evidence-catalog.mjs";
import type { TestEvidenceReport } from "../../.codex/skills/test-evidence-review/scripts/test-evidence-catalog.mjs";

describe("test evidence catalog integration", () => {
  it("accepts a valid test-evidence directory", async () => {
    await withTestWorkspace(async (workspaceRoot) => {
      writeTopicCatalog(workspaceRoot, ["alpha"]);
      writeCase(workspaceRoot, "alpha", "valid.md", validCase("AUX-TEST-CATALOG-VALID-001"));

      const sync = await syncTestEvidenceIndex({ mode: "write", workspaceRoot });
      const report = await validateTestEvidence({ workspaceRoot });

      assert.equal(sync.status, "ok");
      assert.deepEqual(report.diagnostics, []);
      assert.equal(report.summary.testCases, 1);
    });
  });

  it("rejects an unknown topic directory", async () => {
    await withTestWorkspace(async (workspaceRoot) => {
      writeTopicCatalog(workspaceRoot, ["alpha"]);
      writeCase(workspaceRoot, "alpha", "valid.md", validCase("AUX-TEST-CATALOG-VALID-001"));
      writeCase(workspaceRoot, "unknown", "extra.md", validCase("AUX-TEST-CATALOG-EXTRA-001"));

      const report = await validateTestEvidence({ workspaceRoot });

      assertBlockingDiagnostic(report, "catalog.topic-unknown");
    });
  });

  it("rejects an invalid case document", async () => {
    await withTestWorkspace(async (workspaceRoot) => {
      writeTopicCatalog(workspaceRoot, ["alpha"]);
      writeCase(
        workspaceRoot,
        "alpha",
        "invalid.md",
        validCase("AUX-TEST-CATALOG-INVALID-001").replace("Contract:\n- Stable contract.\n\n", "")
      );

      const report = await validateTestEvidence({ workspaceRoot });

      assertBlockingDiagnostic(report, "catalog.invalid");
    });
  });

  it("rejects duplicate case ids across topics", async () => {
    await withTestWorkspace(async (workspaceRoot) => {
      writeTopicCatalog(workspaceRoot, ["alpha", "beta"]);
      const duplicate = validCase("AUX-TEST-CATALOG-DUPLICATE-001");
      writeCase(workspaceRoot, "alpha", "first.md", duplicate);
      writeCase(workspaceRoot, "beta", "second.md", duplicate);

      const report = await validateTestEvidence({ workspaceRoot });

      assertBlockingDiagnostic(report, "catalog.case-id-duplicate");
    });
  });

  it("rejects a stale derived index", async () => {
    await withTestWorkspace(async (workspaceRoot) => {
      writeTopicCatalog(workspaceRoot, ["alpha"]);
      const casePath = writeCase(
        workspaceRoot,
        "alpha",
        "stale.md",
        validCase("AUX-TEST-CATALOG-STALE-001")
      );
      const sync = await syncTestEvidenceIndex({ mode: "write", workspaceRoot });
      assert.equal(sync.status, "ok");

      writeFileSync(casePath, validCase("AUX-TEST-CATALOG-STALE-001").replace(
        "The result is observable.",
        "The changed result is observable."
      ));
      const report = await validateTestEvidence({ workspaceRoot });

      assertBlockingDiagnostic(report, "state-index.index-stale");
    });
  });
});

async function withTestWorkspace(run: (workspaceRoot: string) => Promise<void>): Promise<void> {
  const workspaceRoot = mkdtempSync(path.join(tmpdir(), "docnav-test-evidence-"));
  try {
    await run(workspaceRoot);
  } finally {
    rmSync(workspaceRoot, { force: true, recursive: true });
  }
}

function writeTopicCatalog(workspaceRoot: string, topicIds: readonly string[]): void {
  const evidenceRoot = path.join(workspaceRoot, "docs", "test-evidence");
  mkdirSync(evidenceRoot, { recursive: true });
  writeFileSync(
    path.join(evidenceRoot, "test-evidence-topics.json"),
    `${JSON.stringify({
      schemaVersion: 1,
      topics: topicIds.map((id) => ({
        description: `${id} test responsibility.`,
        id
      }))
    }, null, 2)}\n`
  );
}

function writeCase(
  workspaceRoot: string,
  topic: string,
  fileName: string,
  content: string
): string {
  const topicRoot = path.join(workspaceRoot, "docs", "test-evidence", topic);
  mkdirSync(topicRoot, { recursive: true });
  const casePath = path.join(topicRoot, fileName);
  writeFileSync(casePath, content);
  return casePath;
}

function validCase(id: string): string {
  return [
    `### Case ${id}: Valid case`,
    "",
    "Entry:",
    "- `tests/example.test.ts > valid case`",
    "",
    "Contract:",
    "- Stable contract.",
    "",
    "Proves:",
    "- The result is observable.",
    ""
  ].join("\n");
}

function assertBlockingDiagnostic(report: TestEvidenceReport, code: string): void {
  assert.ok(
    report.diagnostics.some((diagnostic) => diagnostic.blocking && diagnostic.code === code),
    `expected blocking diagnostic ${code}: ${JSON.stringify(report.diagnostics)}`
  );
}
