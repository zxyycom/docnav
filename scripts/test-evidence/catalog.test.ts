import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

import {
  listTestCaseTopics,
  loadTestCaseCatalog,
  queryTestCases,
  showTestCase
} from "./cases.ts";
import {
  assertDiagnostic,
  bunEntity,
  cargoEntity,
  caseDirectory,
  createCaseFixture,
  createFixtureRoot,
  smokeEntity,
  writeTopicFile,
  writeTopics
} from "./fixtures/catalog.ts";
import { workspaceRoot } from "./profile.ts";

test("parses and queries topic-grouped semantic Cases", () => {
  using fixture = createCaseFixture();
  assertCatalogQueries(fixture.root);
  assertCatalogCliQueries(fixture.root);
  assertCatalogCliFailureBoundaries(fixture.root);
});

test("diagnoses malformed Case structure and stable identity conflicts", () => {
  assertMalformedCatalogDiagnostics();
  assertCaseDirectoryLayoutDiagnostics();
  assertCaseSourceSymlinkDiagnostics();
});

function assertCatalogQueries(root: string): void {
  const catalog = loadTestCaseCatalog({ workspaceRoot: root });
  const topics = listTestCaseTopics({ workspaceRoot: root });
  const byTopic = queryTestCases({
    workspaceRoot: root,
    topic: "contract"
  });
  const byEntity = queryTestCases({
    workspaceRoot: root,
    entityKey: bunEntity
  });
  const byOwnerText = queryTestCases({
    workspaceRoot: root,
    ownerRef: "docs/owner.md#contract",
    query: "state unchanged",
    offset: 0,
    limit: 10
  });
  const shown = showTestCase({
    workspaceRoot: root,
    id: "CASE-CONTRACT-REJECT-001"
  });

  assert.deepEqual(catalog.diagnostics, []);
  assert.deepEqual(
    catalog.cases.map(({ id, topic }) => ({ id, topic })),
    [
      { id: "CASE-CONTRACT-REJECT-001", topic: "contract" },
      { id: "CASE-CONTRACT-STATE-002", topic: "contract" },
      { id: "CASE-NAVIGATION-DISPATCH-001", topic: "navigation" }
    ]
  );
  assert.deepEqual(
    topics.topics.map(({ id, cases }) => ({ id, cases })),
    [
      { id: "contract", cases: 2 },
      { id: "navigation", cases: 1 }
    ]
  );
  assert.equal(byTopic.total, 2);
  assert.deepEqual(
    byEntity.items.map(({ id }) => id),
    ["CASE-CONTRACT-REJECT-001", "CASE-CONTRACT-STATE-002"]
  );
  assert.deepEqual(
    byOwnerText.items.map(({ id }) => id),
    ["CASE-CONTRACT-STATE-002"]
  );
  assert.equal(shown.status, "ok");
  assert.equal(shown.item?.title, "Invalid input remains rejected");
}

function assertCatalogCliQueries(root: string): void {
  const cliTopicsJson = runSuccessfulJsonCli<{
    status: string;
    topics: Array<{ id: string; cases: number }>;
  }>(["topics", "--root", root]);
  assert.equal(cliTopicsJson.status, "ok");
  assert.deepEqual(
    cliTopicsJson.topics.map(({ id, cases }) => ({ id, cases })),
    [
      { id: "contract", cases: 2 },
      { id: "navigation", cases: 1 }
    ]
  );

  const cliListJson = runSuccessfulJsonCli<{
    total: number;
    items: Array<{ id: string }>;
  }>([
    "list",
    "--entity-key",
    cargoEntity,
    "--root",
    root
  ]);
  assert.equal(cliListJson.total, 1);
  assert.deepEqual(
    cliListJson.items.map(({ id }) => id),
    ["CASE-CONTRACT-REJECT-001"]
  );

  const cliShowJson = runSuccessfulJsonCli<{
    status: string;
    item: { id: string } | null;
  }>([
    "show",
    "CASE-CONTRACT-REJECT-001",
    "--root",
    root
  ]);
  assert.equal(cliShowJson.status, "ok");
  assert.equal(cliShowJson.item?.id, "CASE-CONTRACT-REJECT-001");
}

function assertCatalogCliFailureBoundaries(root: string): void {
  const cliMissing = runCli([
    "show",
    "CASE-MISSING-001",
    "--root",
    root
  ]);
  assert.equal(cliMissing.status, 6);
  assert.equal(cliMissing.stderr, "");
  const cliMissingJson = JSON.parse(cliMissing.stdout) as {
    status: string;
    diagnostics: Array<{ code: string }>;
    item: null;
  };
  assert.equal(cliMissingJson.status, "error");
  assert.equal(cliMissingJson.item, null);
  assert.ok(
    cliMissingJson.diagnostics.some(({ code }) => (
      code === "query.case-not-found"
    ))
  );

  const cliCheckFailure = runCli([
    "check",
    "--root",
    root
  ]);
  assert.equal(cliCheckFailure.status, 3);
  assert.equal(cliCheckFailure.stdout, "");
  assert.match(
    cliCheckFailure.stderr,
    /profile:runner-profile-invalid:/
  );

  const rejectedCommands = [
    ["sync", "--root", root],
    ["changes", "--root", root],
    ["list", "--entry-key", bunEntity, "--root", root],
    ["list", "--claim-id", "CLAIM-001", "--root", root],
    ["list", "--kind", "entry", "--root", root],
    ["list", "--case-id", "CASE-CONTRACT-REJECT-001", "--root", root]
  ];
  for (const args of rejectedCommands) {
    const rejected = runCli(args);
    assert.equal(rejected.status, 2, args.join(" "));
    assert.equal(rejected.stdout, "", args.join(" "));
    assert.notEqual(rejected.stderr, "", args.join(" "));
  }
}

function assertMalformedCatalogDiagnostics(): void {
  using fixture = createFixtureRoot();
  writeTopics(fixture.root, ["contract", "empty", "other"]);
  writeTopicFile(fixture.root, "contract", [
    "# contract",
    "",
    "## Case CASE-DUPLICATE-001: Missing required semantics",
    "Entities:",
    `- \`${bunEntity}\``,
    `- \`${bunEntity}\``,
    "Proves:",
    ""
  ]);
  writeTopicFile(fixture.root, "empty", ["# empty"]);
  writeTopicFile(fixture.root, "other", [
    "# mismatched",
    "",
    "Case prose outside a Case block is not allowed.",
    "",
    "## Case CASE-DUPLICATE-001: Duplicate identity",
    "Owner: `docs/owner.md#guide--install`",
    "Entities:",
    `- \`${cargoEntity}\``,
    "Proves:",
    "- The public error remains observable.",
    "",
    "## Case CASE-FRONTMATTER-001: Frontmatter is not heading content",
    "Owner: `docs/owner.md#frontmatter-heading`",
    "Entities:",
    `- \`${cargoEntity}\``,
    "Proves:",
    "- Document frontmatter does not create an Owner heading.",
    "",
    "## Case CASE-EMPTY-001: No implementation entity",
    "Owner: `docs/missing.md#contract`",
    "Entities:",
    "Proves:",
    "- The public result remains observable.",
    "",
    "## Case CASE-TYPO-001 Missing the required colon",
    "Owner: `docs/owner.md#contract`",
    "",
    "## Notes",
    "Topic notes are not a Case block.",
    ""
  ]);
  writeTopicFile(fixture.root, "unknown", [
    "# unknown",
    "",
    "## Case CASE-UNKNOWN-SHOULD-NOT-LOAD-001: Unknown topics are not sources",
    "Owner: `docs/owner.md#contract`",
    "Entities:",
    `- \`${smokeEntity}\``,
    "Proves:",
    "- Unknown Markdown cannot contribute semantic Cases.",
    ""
  ]);

  const catalog = loadTestCaseCatalog({ workspaceRoot: fixture.root });

  assertDiagnostic(catalog.diagnostics, "topic.unknown");
  assertDiagnostic(
    catalog.diagnostics,
    "topic.unknown",
    { path: "docs/testing/cases/unknown.md" }
  );
  assertDiagnostic(catalog.diagnostics, "topic.heading-invalid");
  assertDiagnostic(catalog.diagnostics, "topic.content-unexpected");
  assertDiagnostic(catalog.diagnostics, "case.heading-invalid");
  assertDiagnostic(catalog.diagnostics, "topic.heading-unexpected");
  assertDiagnostic(catalog.diagnostics, "case.id-duplicate");
  assertDiagnostic(catalog.diagnostics, "case.owner-missing");
  assertDiagnostic(catalog.diagnostics, "case.owner-unknown");
  assertDiagnostic(catalog.diagnostics, "case.owner-heading-unknown");
  assertDiagnostic(catalog.diagnostics, "case.entity-duplicate");
  assertDiagnostic(catalog.diagnostics, "case.entities-empty");
  assertDiagnostic(catalog.diagnostics, "case.proves-empty");
  assertDiagnostic(
    catalog.diagnostics,
    "case.owner-heading-unknown",
    { caseId: "CASE-DUPLICATE-001" }
  );
  assertDiagnostic(
    catalog.diagnostics,
    "case.owner-heading-unknown",
    { caseId: "CASE-FRONTMATTER-001" }
  );
  assert.equal(
    catalog.cases.some(({ id }) => id === "CASE-UNKNOWN-SHOULD-NOT-LOAD-001"),
    false
  );
  assert.equal(
    catalog.diagnostics.some(({ path: sourcePath }) => (
      sourcePath === "docs/testing/cases/empty.md"
    )),
    false,
    "an H1-only topic is a valid empty topic"
  );
}

function assertCaseDirectoryLayoutDiagnostics(): void {
  using layoutFixture = createCaseFixture();
  const layoutCasesPath = caseDirectory(layoutFixture.root);
  fs.mkdirSync(path.join(layoutCasesPath, "nested"));
  fs.writeFileSync(path.join(layoutCasesPath, "notes.txt"), "not a Case source\n");
  fs.symlinkSync(
    "contract.md",
    path.join(layoutCasesPath, "linked.md"),
    "file"
  );
  const layoutCatalog = loadTestCaseCatalog({
    workspaceRoot: layoutFixture.root
  });
  assertDiagnostic(
    layoutCatalog.diagnostics,
    "cases.nested-directory",
    { path: "docs/testing/cases/nested" }
  );
  assertDiagnostic(
    layoutCatalog.diagnostics,
    "cases.symlink-unsupported",
    { path: "docs/testing/cases/linked.md" }
  );
  assert.equal(
    layoutCatalog.diagnostics.some(({ path: sourcePath }) => (
      sourcePath === "docs/testing/cases/notes.txt"
    )),
    false,
    "unrelated regular non-Markdown files are not Case sources"
  );
}

function assertCaseSourceSymlinkDiagnostics(): void {
  using topicsLinkFixture = createCaseFixture();
  const topicsPath = path.join(caseDirectory(topicsLinkFixture.root), "topics.json");
  fs.renameSync(
    topicsPath,
    path.join(caseDirectory(topicsLinkFixture.root), "topics-source.json")
  );
  fs.symlinkSync("topics-source.json", topicsPath, "file");
  assertDiagnostic(
    loadTestCaseCatalog({ workspaceRoot: topicsLinkFixture.root }).diagnostics,
    "topics.invalid",
    { path: "docs/testing/cases/topics.json" }
  );

  using topicLinkFixture = createCaseFixture();
  const topicPath = path.join(caseDirectory(topicLinkFixture.root), "contract.md");
  fs.renameSync(
    topicPath,
    path.join(caseDirectory(topicLinkFixture.root), "contract-source.txt")
  );
  fs.symlinkSync("contract-source.txt", topicPath, "file");
  assertDiagnostic(
    loadTestCaseCatalog({ workspaceRoot: topicLinkFixture.root }).diagnostics,
    "cases.symlink-unsupported",
    { path: "docs/testing/cases/contract.md" }
  );

  using rootLinkFixture = createCaseFixture();
  const rootCasesPath = caseDirectory(rootLinkFixture.root);
  fs.renameSync(rootCasesPath, `${rootCasesPath}-source`);
  fs.symlinkSync(
    path.basename(`${rootCasesPath}-source`),
    rootCasesPath,
    "dir"
  );
  assertDiagnostic(
    loadTestCaseCatalog({ workspaceRoot: rootLinkFixture.root }).diagnostics,
    "cases.directory-invalid",
    { path: "docs/testing/cases" }
  );
}

function runCli(args: readonly string[]): {
  status: number | null;
  stdout: string;
  stderr: string;
} {
  const result = spawnSync(
    process.execPath,
    [
      path.join(workspaceRoot, "scripts", "test-evidence", "index.ts"),
      ...args
    ],
    {
      cwd: workspaceRoot,
      encoding: "utf8"
    }
  );
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  return {
    status: result.status,
    stdout: result.stdout,
    stderr: result.stderr
  };
}

function runSuccessfulJsonCli<T>(args: readonly string[]): T {
  const result = runCli(args);
  assert.equal(result.status, 0);
  assert.equal(result.stderr, "");
  return JSON.parse(result.stdout) as T;
}
