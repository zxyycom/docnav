import { describe, it } from "node:test";
import assert from "node:assert/strict";

import { validateCaseCatalogSnapshot } from "./index.ts";

// @case AUX-CASE-CATALOG-001
describe("case catalog validator", () => {
  it("accepts implemented and planned cases with matching source markers", () => {
    const failures = validateCaseCatalogSnapshot({
      catalogText: caseCatalogText([
        implementedCase("WB-CORE-GOOD-001", "crates/good.rs"),
        plannedCase("AUX-CASE-FUTURE-001")
      ]),
      markers: [{ id: "WB-CORE-GOOD-001", relPath: "crates/good.rs" }]
    });

    assert.deepEqual(failures, []);
  });

  it("reports status, planned marker, duplicate marker, and path drift", () => {
    const failures = validateCaseCatalogSnapshot({
      catalogText: caseCatalogText([
        implementedCase("WB-CORE-GOOD-001", "crates/good.rs"),
        implementedCase("WB-CORE-NOCODE-001", null),
        invalidStatusCase("WB-CORE-BADSTATUS-001"),
        implementedCase("WB-CORE-DUP-001", "crates/dup.rs"),
        implementedCase("WB-CORE-DUP-001", "crates/dup.rs"),
        plannedCase("AUX-CASE-FUTURE-001"),
        implementedCase("WB-CORE-PATH-001", "crates/expected.rs"),
        implementedCase("WB-CORE-MISSING-001", "crates/missing.rs")
      ]),
      markers: [
        { id: "WB-CORE-GOOD-001", relPath: "crates/good.rs" },
        { id: "WB-CORE-GOOD-001", relPath: "crates/good.rs" },
        { id: "AUX-CASE-FUTURE-001", relPath: "scripts/future.test.ts" },
        { id: "WB-CORE-PATH-001", relPath: "crates/actual.rs" },
        { id: "WB-CORE-EXTRA-001", relPath: "crates/extra.rs" },
        { id: "NOT-A-CASE", relPath: "crates/bad.rs" }
      ]
    });

    assertFailure(failures, "duplicate case IDs in docs/testing/cases.md");
    assertFailure(failures, "duplicate source @case markers");
    assertFailure(failures, "source @case markers must use CATEGORY-SCOPE-INTENT-NNN");
    assertFailure(failures, "documented cases must declare Status: implemented or Status: planned");
    assertFailure(failures, "implemented documented cases must declare Code");
    assertFailure(failures, "documented case IDs missing @case source markers");
    assertFailure(failures, "source @case markers missing from docs/testing/cases.md");
    assertFailure(failures, "planned cases must not have source @case markers yet");
    assertFailure(failures, "documented Code paths must match source @case marker paths");
  });
});

function caseCatalogText(entries: readonly string[]): string {
  return ["# Test Cases", "", ...entries].join("\n");
}

function implementedCase(id: string, codePath: string | null): string {
  return [
    `### ${id} Implemented case`,
    "Status: implemented",
    ...(codePath === null ? [] : [`Code: \`${codePath}\``]),
    "",
    "Proves:",
    "- test case"
  ].join("\n");
}

function plannedCase(id: string): string {
  return [`### ${id} Planned case`, "Status: planned", "", "Proves:", "- future test"].join("\n");
}

function invalidStatusCase(id: string): string {
  return [`### ${id} Invalid status`, "Status: done", "", "Proves:", "- invalid status"].join("\n");
}

function assertFailure(failures: readonly string[], label: string): void {
  assert.ok(
    failures.some((failure) => failure.includes(label)),
    `expected failure label ${label}\n${failureReport(failures)}`
  );
}

function failureReport(failures: readonly string[]): string {
  return failures.join("\n\n");
}
