import fs from "node:fs";
import path from "node:path";

import { runCli, runProtocolResponseCase, runSuccessfulJsonCase, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectNoProtocolEnvelope,
  expectObjectArray,
  expectStderrEmpty,
  expectStdoutIncludes,
  expectString,
  parseJson
} from "../assertions.ts";
import type { CommandRecord, SmokeCommandOptions } from "../../../tools/smoke-harness.ts";
import {
  createProject,
  type MarkdownConfigProject,
  writeJson,
  writeProjectConfig,
  writeUserConfig
} from "./config-fixtures.ts";

export function createMarkdownConfigTasks() {
  return [
    // @case BB-MD-CONFIG-001
    {
      id: "MD-CONFIG-001",
      label: "MD-CONFIG-001 direct CLI config precedence and boundaries",
      run: testMarkdownDirectCliConfig
    }
  ];
}

async function testMarkdownDirectCliConfig() {
  const project = createProject("direct-config");
  writeProjectConfig(project, {
    defaults: {
      limit: 40,
      output: "readable-json"
    },
    options: {
      max_heading_level: 1
    }
  });
  writeUserConfig(project, {
    defaults: {
      limit: 20,
      output: "protocol-json"
    },
    options: {
      max_heading_level: 2
    }
  });

  await assertProjectConfigApplies(project);
  await assertExplicitArgvOverridesConfig(project);
  await assertProjectConfigPathOverrideReplacesDefault(project);
  await assertSkippedConfigSourceWarning(project);
  await assertInvokeIgnoresDirectCliConfig(project);
  await assertHelpAndMachineCommandsDoNotReadConfig();
}

async function assertProjectConfigApplies(project: MarkdownConfigProject) {
  const { record: outlineRecord, json: outline } = await runReadableOutline(
    project,
    "MD-CONFIG-001 outline uses project config defaults",
    ["outline", project.docRelPath]
  );
  const entries = expectObjectArray(outlineRecord, outline.entries, "project-config outline entries are objects");
  expect(outlineRecord, entries.length === 1, "project max_heading_level limits outline to one entry");
  expectRefLevel(outlineRecord, entries[0]?.ref, 1, "project config outline ref");

  const { record: readRecord, json: read } = await runSuccessfulJsonCase(
    "MD-CONFIG-001 read uses configured limit and output",
    ["read", project.docRelPath, "--ref", "doc:full"],
    {
      commandOptions: cwd(project),
      schema: "readableRead",
      check: (record, json) => {
        expectNoProtocolEnvelope(record, json);
        const content = expectString(record, json.content, "read content is string");
        expect(record, typeof json.page === "number", "project limit produces continuation page");
        expect(record, content.length < project.docText.length, "configured limit truncates read content");
      }
    }
  );
  expect(readRecord, read.page === 2, "project limit starts read continuation at page 2");

  const { record: findRecord, json: find } = await runSuccessfulJsonCase(
    "MD-CONFIG-001 find uses configured max heading level",
    ["find", project.docRelPath, "--query", "deep-target"],
    {
      commandOptions: cwd(project),
      schema: "readableFind",
      check: (record, json) => {
        expectNoProtocolEnvelope(record, json);
        const matches = expectObjectArray(record, json.matches, "find matches are objects");
        expect(record, matches.length > 0, "find returns configured match");
        expectRefLevel(record, matches[0]?.ref, 1, "project config find ref");
      }
    }
  );
  expect(findRecord, Array.isArray(find.matches), "find result keeps matches array");
}

async function assertExplicitArgvOverridesConfig(project: MarkdownConfigProject) {
  const { record, json } = await runReadableOutline(
    project,
    "MD-CONFIG-001 explicit max heading level overrides config",
    [
      "outline",
      project.docRelPath,
      "--max-heading-level",
      "3",
      "--limit",
      "6000",
      "--output",
      "readable-json"
    ]
  );
  assertOutlineHasRefLevels(record, json, [1, 2, 3], "explicit max heading level outline");

  await runSuccessfulJsonCase(
    "MD-CONFIG-001 explicit limit overrides config",
    ["read", project.docRelPath, "--ref", "doc:full", "--limit", "6000", "--output", "readable-json"],
    {
      commandOptions: cwd(project),
      schema: "readableRead",
      check: (record, json) => {
        expectNoProtocolEnvelope(record, json);
        expect(record, json.page === null, "explicit limit reads full document in one page");
      }
    }
  );

  await runSuccessfulJsonCase(
    "MD-CONFIG-001 explicit find max heading level overrides config",
    [
      "find",
      project.docRelPath,
      "--query",
      "deep-target",
      "--max-heading-level",
      "3",
      "--output",
      "readable-json"
    ],
    {
      commandOptions: cwd(project),
      schema: "readableFind",
      check: (record, json) => {
        const matches = expectObjectArray(record, json.matches, "find matches are objects");
        expectRefLevel(record, matches[0]?.ref, 3, "explicit max heading level find ref");
      }
    }
  );
}

async function assertProjectConfigPathOverrideReplacesDefault(project: MarkdownConfigProject) {
  const overridePath = path.join(project.fixturesDir, "project-override.json");
  writeJson(overridePath, {
    defaults: {
      limit: 6000,
      output: "readable-json"
    },
    options: {
      max_heading_level: 3
    }
  });

  const { record, json } = await runReadableOutline(
    project,
    "MD-CONFIG-001 project config path override replaces default",
    ["outline", project.docRelPath, "--project-config-path", "fixtures/project-override.json"]
  );

  assertOutlineHasRefLevels(record, json, [1, 2, 3], "project override outline");
}

async function assertSkippedConfigSourceWarning(project: MarkdownConfigProject) {
  const record = await runCli(
    "MD-CONFIG-001 missing project override emits readable warning",
    [
      "outline",
      project.docRelPath,
      "--project-config-path",
      "fixtures/missing-project.json",
      "--output",
      "readable-json"
    ],
    cwd(project)
  );
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "readableOutline", json);
  const warnings = expectObjectArray(record, json.warnings, "config warning array exists");
  const warning = expectJsonObject(record, warnings[0], "config warning is object");
  const details = expectJsonObject(record, warning.details, "config warning details is object");
  expect(record, warning.id === "adapter_config_source_skipped", "config warning id matches");
  expect(record, warning.effect === "operation_continued", "config warning effect matches");
  expect(record, details.source_level === "project", "config warning source_level is project");
  expect(record, details.path_origin === "override", "config warning path_origin is override");
  expect(record, details.reason_code === "missing_override", "config warning reason_code is missing_override");
  expect(
    record,
    typeof details.path === "string" && details.path.includes("missing-project.json"),
    "config warning path identifies attempted override"
  );
}

async function assertInvokeIgnoresDirectCliConfig(project: MarkdownConfigProject) {
  const request = {
    protocol_version: "0.1",
    request_id: "smoke-config-invoke",
    operation: "outline",
    document: { path: project.docRelPath },
    arguments: {
      limit: 6000,
      page: 1
    }
  };

  await runProtocolResponseCase("MD-CONFIG-001 invoke ignores direct CLI config", ["invoke"], {
    commandOptions: {
      ...cwd(project),
      stdin: JSON.stringify(request),
      stdinSummary: "outline request without options"
    },
    operation: "outline",
    check: (record, json) => {
      const result = expectJsonObject(record, json.result, "invoke outline result is object");
      const entries = expectObjectArray(record, result.entries, "invoke outline entries are objects");
      expect(record, entries.length === 3, "invoke uses request/default options instead of direct CLI config");
      assertEntryLevels(record, entries, [1, 2, 3], "invoke outline");
    }
  });
}

async function assertHelpAndMachineCommandsDoNotReadConfig() {
  const project = createProject("invalid-config-boundary");
  fs.writeFileSync(path.join(project.docnavDir, "docnav-markdown.json"), "{ invalid json", "utf8");

  const help = await runCli("MD-CONFIG-001 help does not read config", ["outline", "--help"], cwd(project));
  expectExit(help, 0);
  expectStderrEmpty(help);
  expectStdoutIncludes(help, "--project-config-path <path>");
  expectStdoutIncludes(help, "--user-config-path <path>");

  await runSuccessfulJsonCase("MD-CONFIG-001 manifest ignores invalid config", ["manifest", "--output", "protocol-json"], {
    commandOptions: cwd(project),
    schema: "manifest"
  });

  await runSuccessfulJsonCase(
    "MD-CONFIG-001 probe ignores invalid config",
    ["probe", project.docRelPath, "--output", "protocol-json"],
    {
      commandOptions: cwd(project),
      schema: "probe"
    }
  );
}

async function runReadableOutline(project: MarkdownConfigProject, name: string, args: string[]) {
  return runSuccessfulJsonCase(name, args, {
    commandOptions: cwd(project),
    schema: "readableOutline",
    check: expectNoProtocolEnvelope
  });
}

function assertOutlineHasRefLevels(
  record: CommandRecord,
  json: Record<string, unknown>,
  expectedLevels: readonly number[],
  label: string
) {
  const entries = expectObjectArray(record, json.entries, `${label} entries are objects`);
  expect(record, entries.length === expectedLevels.length, `${label} has expected entry count`);
  assertEntryLevels(record, entries, expectedLevels, label);
}

function assertEntryLevels(
  record: CommandRecord,
  entries: readonly unknown[],
  expectedLevels: readonly number[],
  label: string
) {
  for (const [index, expectedLevel] of expectedLevels.entries()) {
    const entry = expectJsonObject(record, entries[index], `${label} entry ${index + 1} is object`);
    expectRefLevel(record, entry.ref, expectedLevel, `${label} entry ${index + 1}`);
  }
}

function expectRefLevel(record: CommandRecord, ref: unknown, level: number, label: string) {
  const value = expectString(record, ref, `${label} ref is string`);
  expect(record, value.endsWith(`:H${level}`), `${label} ref uses H${level}`);
}

function cwd(project: MarkdownConfigProject): SmokeCommandOptions {
  return { cwd: project.root };
}
