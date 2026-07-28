import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  listTestCaseTopics,
  loadTestCaseCatalog,
  queryTestCases,
  showTestCase,
  validateTestCaseCoverage
} from "./cases.ts";
import { exitCodeForDiagnostics } from "./cli.ts";
import { closeStaticAndRuntimeEntities } from "./closure.ts";
import { discoverTestEntities } from "./discover.ts";
import { parseBunJUnit } from "./discovery/bun.ts";
import { resolveBunTestFiles } from "./discovery/bun-files.ts";
import { parseLibtestList } from "./discovery/rust.ts";
import {
  diagnostic,
  type RuntimeTestEntity,
  type StaticTestEntity,
  type TestEntity,
  type TestEvidenceDiagnostic
} from "./model.ts";
import {
  loadSupportedRunnerProfile,
  workspaceRoot
} from "./profile.ts";

const bunEntity = "bun|tests/example.test.ts|contract > rejects invalid input";
const cargoEntity = "cargo|example:lib:example|tests::rejects_invalid_input";
const smokeEntity = "smoke|core:root|CORE-CONTRACT-001";

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

test("closes current test entities against the union of Case mappings", () => {
  assertCaseMappingClosure();
  assertStaticRuntimeClosure();
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

function assertCaseMappingClosure(): void {
  using fixture = createCaseFixture();
  const catalog = loadTestCaseCatalog({ workspaceRoot: fixture.root });
  const entities = [
    testEntity(bunEntity),
    testEntity(cargoEntity),
    testEntity(smokeEntity)
  ];

  assert.deepEqual(validateTestCaseCoverage({ catalog, entities }), []);

  const unknownEntity = "bun|tests/missing.test.ts|missing";
  const changedCatalog = {
    ...catalog,
    cases: catalog.cases.map((testCase) => (
      testCase.id === "CASE-CONTRACT-REJECT-001"
        ? {
            ...testCase,
            entityKeys: testCase.entityKeys.map((entityKey) => (
              entityKey === cargoEntity ? unknownEntity : entityKey
            ))
          }
        : testCase
    ))
  };
  const diagnostics = validateTestCaseCoverage({
    catalog: changedCatalog,
    entities
  });

  assertDiagnostic(diagnostics, "case.entity-unknown");
  assertDiagnostic(diagnostics, "entity.case-missing");
  assert.equal(
    diagnostics.some(({ code, entityKey }) => (
      code === "entity.case-missing" && entityKey === bunEntity
    )),
    false,
    "one entity may be mapped by multiple Cases"
  );
}

function assertStaticRuntimeClosure(): void {
  const identity = "tests/example.test.ts\0contract > rejects invalid input";
  const staticEntity = staticTestEntity(identity);
  const runtimeEntity = runtimeTestEntity(identity);
  const closed = closeBunEntities([staticEntity], [runtimeEntity]);
  assert.deepEqual(closed.diagnostics, []);
  assert.deepEqual(
    closed.entities.map(({ entityKey }) => entityKey),
    [bunEntity]
  );
  assertUnmatchedEntityClosure(staticEntity, runtimeEntity);
  assertDuplicateEntityClosure(staticEntity, runtimeEntity);
}

function assertUnmatchedEntityClosure(
  staticEntity: StaticTestEntity,
  runtimeEntity: RuntimeTestEntity
): void {
  const staticOnly = closeBunEntities([staticEntity], []);
  assertDiagnostic(staticOnly.diagnostics, "static-only");
  assert.match(staticOnly.diagnostics[0]?.message ?? "", /static TestEntity/);

  const runtimeOnly = closeBunEntities([], [runtimeEntity]);
  assertDiagnostic(runtimeOnly.diagnostics, "runtime-only");
  assert.match(runtimeOnly.diagnostics[0]?.message ?? "", /runtime TestEntity/);
}

function assertDuplicateEntityClosure(
  staticEntity: StaticTestEntity,
  runtimeEntity: RuntimeTestEntity
): void {
  const duplicateStatic = closeBunEntities(
    [staticEntity, { ...staticEntity }],
    [runtimeEntity]
  );
  assertDiagnostic(duplicateStatic.diagnostics, "duplicate-entity");
  assert.equal(duplicateStatic.diagnostics[0]?.origin, "static");
  assert.match(
    duplicateStatic.diagnostics[0]?.message ?? "",
    /TestEntity identity/
  );

  const duplicateRuntime = closeBunEntities(
    [staticEntity],
    [runtimeEntity, { ...runtimeEntity }]
  );
  assertDiagnostic(duplicateRuntime.diagnostics, "duplicate-entity");
  assert.equal(duplicateRuntime.diagnostics[0]?.origin, "runner");
  assert.match(
    duplicateRuntime.diagnostics[0]?.message ?? "",
    /TestEntity identity/
  );
}

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

test("uses distinct exit statuses for discovery, runner, Case, and query failures", () => {
  assert.equal(exitCodeForDiagnostics([
    diagnostic("unsupported-entity-shape", "static", "unsupported")
  ]), 3);
  assert.equal(exitCodeForDiagnostics([
    diagnostic("runner-report-failed", "runner", "failed")
  ]), 4);
  assert.equal(exitCodeForDiagnostics([
    diagnostic("entity.case-missing", "case", "missing")
  ]), 5);
  assert.equal(exitCodeForDiagnostics([
    diagnostic("query.case-not-found", "query", "unknown")
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

  const rootMismatch = await discoverTestEntities({
    workspaceRoot: os.tmpdir()
  });
  assert.ok(rootMismatch.diagnostics.some(({ code, message }) => (
    code === "runner-profile-invalid" &&
    message.includes("current checkout")
  )));
});

function createCaseFixture(): Fixture {
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

function createFixtureRoot(): Fixture {
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

function caseDirectory(root: string): string {
  return path.join(root, "docs", "testing", "cases");
}

function writeTopics(root: string, topicIds: readonly string[]): void {
  writeJson(
    path.join(root, "docs", "testing", "cases", "topics.json"),
    {
      schemaVersion: 1,
      topics: topicIds.map((id) => ({
        id,
        description: `${id} behavior.`
      }))
    }
  );
}

function writeTopicFile(
  root: string,
  topic: string,
  lines: readonly string[]
): void {
  fs.writeFileSync(
    path.join(root, "docs", "testing", "cases", `${topic}.md`),
    lines.join("\n")
  );
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

function testEntity(entityKey: string): TestEntity {
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

function staticTestEntity(identity: string): StaticTestEntity {
  return {
    identity,
    sourcePath: "tests/example.test.ts",
    sourceRange: {
      startLine: 7,
      startColumn: 1,
      endLine: 7,
      endColumn: 42
    }
  };
}

function runtimeTestEntity(identity: string): RuntimeTestEntity {
  return {
    identity,
    target: "tests/example.test.ts",
    selector: "contract > rejects invalid input"
  };
}

function closeBunEntities(
  statics: StaticTestEntity[],
  runtime: RuntimeTestEntity[]
): ReturnType<typeof closeStaticAndRuntimeEntities> {
  return closeStaticAndRuntimeEntities({
    runner: "bun",
    statics,
    runtime,
    createEntityKey: ({ target, selector }) => `bun|${target}|${selector}`
  });
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

function writeJson(targetPath: string, value: unknown): void {
  fs.writeFileSync(targetPath, `${JSON.stringify(value, null, 2)}\n`);
}

function assertDiagnostic(
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

type Fixture = {
  root: string;
  [Symbol.dispose](): void;
};
