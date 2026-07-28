import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import type {
  TestEntity,
  TestEvidenceDiagnostic
} from "../model.ts";

export const bunEntity =
  "bun|tests/example.test.ts|contract > rejects invalid input";
export const cargoEntity =
  "cargo|example:lib:example|tests::rejects_invalid_input";
export const smokeEntity = "smoke|core:root|CORE-CONTRACT-001";

export type CatalogFixture = {
  root: string;
  [Symbol.dispose](): void;
};

export function createCaseFixture(): CatalogFixture {
  const fixture = createFixtureRoot();
  writeTopics(fixture.root, ["contract", "navigation"]);
  writeTopicFile(fixture.root, "contract", [
    "# contract",
    "",
    "## Case CASE-CONTRACT-REJECT-001: Invalid input remains rejected",
    "Owner: `docs/owner.md#contract`",
    "Entities:",
    `- \`${bunEntity}\``,
    `- \`${cargoEntity}\``,
    "Proves:",
    "- Invalid input returns the public error.",
    "",
    "## Case CASE-CONTRACT-STATE-002: Rejection preserves state",
    "Owner: `docs/owner.md#contract`",
    "Entities:",
    `- \`${bunEntity}\``,
    "Proves:",
    "- The caller observes the protected state unchanged.",
    ""
  ]);
  writeTopicFile(fixture.root, "navigation", [
    "# navigation",
    "",
    "## Case CASE-NAVIGATION-DISPATCH-001: Dispatch selects the requested adapter",
    "Owner: `docs/owner.md#navigation`",
    "Entities:",
    `- \`${smokeEntity}\``,
    "Proves:",
    "- The selected adapter handles the request.",
    ""
  ]);
  return fixture;
}

export function createFixtureRoot(): CatalogFixture {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "docnav-test-cases-"));
  fs.mkdirSync(path.join(root, "docs", "testing", "cases"), {
    recursive: true
  });
  fs.writeFileSync(
    path.join(root, "docs", "owner.md"),
    [
      "---",
      "title: Owner fixture",
      "## Frontmatter Heading",
      "---",
      "# Owner",
      "",
      "```text",
      "# Output excerpt",
      "",
      "## Guide > Install",
      "```",
      "",
      "## Contract",
      "",
      "Invalid input is rejected without changing state.",
      "",
      "## Navigation",
      "",
      "The selected adapter handles the request.",
      ""
    ].join("\n")
  );
  return {
    root,
    [Symbol.dispose]() {
      fs.rmSync(root, { force: true, recursive: true });
    }
  };
}

export function caseDirectory(root: string): string {
  return path.join(root, "docs", "testing", "cases");
}

export function writeTopics(
  root: string,
  topicIds: readonly string[]
): void {
  const document = {
    schemaVersion: 1,
    topics: topicIds.map((id) => ({
      id,
      description: `${id} behavior.`
    }))
  };
  fs.writeFileSync(
    path.join(caseDirectory(root), "topics.json"),
    `${JSON.stringify(document, null, 2)}\n`
  );
}

export function writeTopicFile(
  root: string,
  topic: string,
  lines: readonly string[]
): void {
  fs.writeFileSync(
    path.join(caseDirectory(root), `${topic}.md`),
    lines.join("\n")
  );
}

export function testEntity(entityKey: string): TestEntity {
  const [runner = "bun", target = "tests/example.test.ts", selector = "case"] =
    entityKey.split("|");
  return {
    entityKey,
    runner,
    target,
    selector,
    sourcePath: target,
    sourceRange: {
      startLine: 1,
      startColumn: 1,
      endLine: 1,
      endColumn: 10
    }
  };
}

export function assertDiagnostic(
  diagnostics: readonly TestEvidenceDiagnostic[],
  code: string,
  expected: {
    caseId?: string;
    path?: string;
  } = {}
): void {
  assert.ok(
    diagnostics.some((value) => (
      value.code === code &&
      value.blocking &&
      (expected.caseId === undefined || value.caseId === expected.caseId) &&
      (expected.path === undefined || value.path === expected.path)
    )),
    `expected blocking diagnostic ${code} ${JSON.stringify(expected)}: ` +
    JSON.stringify(diagnostics)
  );
}
