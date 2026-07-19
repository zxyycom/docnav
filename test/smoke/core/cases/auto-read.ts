import {
  configFixturePath,
  configFixtureProject,
  createProject,
  type SmokeProject,
} from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectNoProtocolEnvelope,
  expectNoReadableViewBlocks,
  expectObjectArray,
  expectProtocolSuccess,
  expectReadableViewBlockRestoresField,
  expectStderrEmpty,
  expectString,
  parseJson,
  parseReadableViewHeader,
} from "../assertions.ts";
import type { CommandRecord } from "../../../tools/smoke-harness.ts";
import type { JsonRecord } from "../assertions.ts";

interface AutoReadFacts {
  content: string;
  ref: string;
}

export function createAutoReadTasks() {
  return [
    // @case BB-CORE-AUTO-READ-001
    {
      id: "CORE-AUTO-READ-001",
      label: "CORE-AUTO-READ-001 unique-ref auto-read defaults and disable sources",
      run: testAutoReadDefaultsAndDisableSources,
    },
  ];
}

async function testAutoReadDefaultsAndDisableSources() {
  const defaultProject = createProject("auto-read-default");
  const facts = await assertDefaultProtocolAutoRead(defaultProject);
  await assertDefaultReadableAutoRead(defaultProject, facts);
  await assertCliDisabledReadableBase(defaultProject);

  const projectDisabled = configFixtureProject("auto-read-disabled", "auto-read-project-disabled");
  await assertProtocolAutoReadAbsent(
    "CORE-AUTO-READ-001 project config disabled keeps base find",
    projectDisabled,
  );

  const userDisabled = createProject("auto-read-user-disabled");
  await assertProtocolAutoReadAbsent(
    "CORE-AUTO-READ-001 user config disabled keeps base find",
    userDisabled,
    ["--user-config", configFixturePath("auto-read-disabled")],
  );
}

async function assertDefaultProtocolAutoRead(project: SmokeProject): Promise<AutoReadFacts> {
  const record = await runFind(
    "CORE-AUTO-READ-001 omitted sources default to unique-ref protocol-json",
    project,
    "protocol-json",
  );
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolSuccess(record, json, "find");
  const result = expectJsonObject(record, json.result, "find result is an object");
  const ref = expectUniqueFindRef(record, result);
  const autoRead = expectJsonObject(record, result.auto_read, "default find result includes auto_read");
  expect(record, autoRead.reason === "unique_ref", "auto_read reports unique_ref reason");
  const read = expectJsonObject(record, autoRead.read, "auto_read includes nested read result");
  const nestedRef = expectString(record, read.ref, "nested read ref is a string");
  const content = expectString(record, read.content, "nested read content is a string");
  expect(record, nestedRef === ref, "nested read reuses the unique find ref");
  expect(record, content.includes("## Install"), "nested read returns the matched Markdown section");
  expect(record, read.content_type === "text/markdown", "nested read preserves Markdown content type");
  return { content, ref };
}

async function assertDefaultReadableAutoRead(project: SmokeProject, facts: AutoReadFacts) {
  const record = await runFind(
    "CORE-AUTO-READ-001 omitted sources default to unique-ref readable-view",
    project,
    "readable-view",
  );
  expectExit(record, 0);
  expectStderrEmpty(record);
  const header = parseReadableViewHeader(record);
  expectNoProtocolEnvelope(record, header);
  expectUniqueFindRef(record, header);
  const autoRead = expectJsonObject(record, header.auto_read, "readable find header includes auto_read");
  expect(record, autoRead.reason === "unique_ref", "readable auto_read reports unique_ref reason");
  const read = expectJsonObject(record, autoRead.read, "readable auto_read includes nested read facts");
  expect(record, read.ref === facts.ref, "readable nested read preserves the protocol ref");
  expect(record, read.content_type === "text/markdown", "readable nested read preserves content type");
  expectReadableViewBlockRestoresField(
    record,
    record.stdout,
    "/auto_read/read/content",
    facts.content,
  );
}

async function assertCliDisabledReadableBase(project: SmokeProject) {
  const record = await runFind(
    "CORE-AUTO-READ-001 CLI disabled keeps readable base find",
    project,
    "readable-view",
    ["--auto-read", "disabled"],
  );
  expectExit(record, 0);
  expectStderrEmpty(record);
  const header = parseReadableViewHeader(record);
  expectNoProtocolEnvelope(record, header);
  expectUniqueFindRef(record, header);
  expect(record, !Object.hasOwn(header, "auto_read"), "CLI disabled omits readable auto_read");
  expectNoReadableViewBlocks(record);
}

async function assertProtocolAutoReadAbsent(
  name: string,
  project: SmokeProject,
  extraArgs: string[] = [],
) {
  const record = await runFind(name, project, "protocol-json", extraArgs);
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolSuccess(record, json, "find");
  const result = expectJsonObject(record, json.result, "disabled find result is an object");
  expectUniqueFindRef(record, result);
  expect(record, !Object.hasOwn(result, "auto_read"), "disabled source omits protocol auto_read");
}

function runFind(
  name: string,
  project: SmokeProject,
  output: "protocol-json" | "readable-view",
  extraArgs: string[] = [],
) {
  return runCli(
    name,
    [
      "find",
      project.normalRelPath,
      "--query",
      "Install",
      ...extraArgs,
      "--output",
      output,
    ],
    { project },
  );
}

function expectUniqueFindRef(record: CommandRecord, result: JsonRecord): string {
  const matches = expectObjectArray(record, result.matches, "find matches are objects");
  expect(record, matches.length === 2, "find preserves both matching base entries");
  const refs = matches.map((match, index) =>
    expectString(record, match.ref, `find match ${index + 1} ref is a string`),
  );
  const uniqueRefs = new Set(refs);
  expect(record, uniqueRefs.size === 1, "find base entries share one distinct ref");
  return refs[0] ?? "";
}
