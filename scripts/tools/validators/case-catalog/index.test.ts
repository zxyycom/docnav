import { describe, it } from "node:test";
import assert from "node:assert/strict";

import { caseSourceFiles, collectCaseMarkers } from "./source-markers.ts";
import { validateCaseCatalogSnapshot } from "./validator.ts";

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

  it("discovers case markers in the nested cli-config workspace", () => {
    const markers = collectCaseMarkers(caseSourceFiles());

    assert.deepEqual(markers.byId.get("WB-PARAM-FIELD-CONTRACT-001"), [
      "subrepos/cli-config-resolution/crates/typed-fields/tests/canonical_parameters.rs"
    ]);
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
        implementedCase("WB-CORE-MISSING-001", "crates/missing.rs"),
        missingProvesCase("WB-CORE-NOPROVES-001", "crates/no-proves.rs"),
        multipleCodeCase("WB-CORE-MULTICODE-001", "crates/multi.rs", "crates/extra.rs")
      ]),
      markers: [
        { id: "WB-CORE-GOOD-001", relPath: "crates/good.rs" },
        { id: "WB-CORE-GOOD-001", relPath: "crates/good.rs" },
        { id: "AUX-CASE-FUTURE-001", relPath: "scripts/future.test.ts" },
        { id: "WB-CORE-PATH-001", relPath: "crates/actual.rs" },
        { id: "WB-CORE-NOPROVES-001", relPath: "crates/no-proves.rs" },
        { id: "WB-CORE-MULTICODE-001", relPath: "crates/multi.rs" },
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
    assertFailure(failures, "documented cases must include non-empty Proves");
    assertFailure(failures, "documented cases must declare exactly one Code path");
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

function missingProvesCase(id: string, codePath: string): string {
  return [`### ${id} Missing proves`, "Status: implemented", `Code: \`${codePath}\``].join("\n");
}

function multipleCodeCase(id: string, codePath: string, extraPath: string): string {
  return [
    `### ${id} Multiple code paths`,
    "Status: implemented",
    `Code: \`${codePath}\`, \`${extraPath}\``,
    "",
    "Proves:",
    "- malformed code declaration"
  ].join("\n");
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
