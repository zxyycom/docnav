import { createProject, createRealMarkdownAdapter, writeRegistry } from "../fixtures.ts";
import type { SmokeProject } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectNoJsonPayloadInStderr,
  expectObjectArray,
  expectProtocolFailure,
  expectString,
  expectStringArray,
  parseJson
} from "../assertions.ts";
import {
  assertReadableReadRefHandoff,
  runReadableJsonCase
} from "./readable-output.ts";

export function createRealMarkdownLinkTasks() {
  return [
    // @case BB-CORE-LINK-001
    {
      id: "CORE-LINK-001",
      label: "CORE-LINK-001 real markdown ref handoff chain",
      run: testRealMarkdownRefHandoffChain
    }
  ];
}

export function createRealMarkdownRefErrorTasks() {
  return [
    // @case BB-CORE-REF-001
    {
      id: "CORE-REF-001",
      label: "CORE-REF-001 real markdown ref error mapping",
      run: testRealMarkdownRefInvalidProtocol
    }
  ];
}

async function testRealMarkdownRefHandoffChain() {
  const project = createRegisteredRealMarkdownProject("real-markdown-ref-handoff");

  const outlineRef = await readFirstOutlineRef(project);
  await assertReadableReadRefHandoff(
    project,
    "CORE-LINK-001 read outline ref readable-json",
    project.normalRelPath,
    outlineRef,
    {
      contentIncludes: "# Guide",
      contentIncludesSummary: "read content includes Markdown heading",
      contentSummary: "outline read content is a string",
      contentType: {
        summary: "read preserves content_type",
        value: "text/markdown"
      },
      refSummary: "read preserves outline ref"
    }
  );

  const findRef = await readFirstFindRef(project);
  await assertReadableReadRefHandoff(
    project,
    "CORE-LINK-001 read find ref readable-json",
    project.normalRelPath,
    findRef,
    {
      contentIncludes: "## Install",
      contentIncludesSummary: "read content includes Install heading",
      contentSummary: "find read content is a string",
      refSummary: "read preserves find ref"
    }
  );

  await assertInfoReadableOutput(project);
}

async function testRealMarkdownRefInvalidProtocol() {
  const project = createRegisteredRealMarkdownProject("real-markdown-ref-invalid-protocol");

  const record = await runCli("CORE-REF-001 ref_invalid protocol-json", [
    "read",
    project.normalRelPath,
    "--ref",
    "bad:ref",
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, 3);
  expectNoJsonPayloadInStderr(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolFailure(record, json, "read", "REF_INVALID");
  const error = expectJsonObject(record, json.error, "protocol error is an object");
  const details = expectJsonObject(record, error.details, "protocol error details is an object");
  expect(record, Object.hasOwn(details, "ref"), "REF_INVALID includes details.ref");
  expect(record, Object.hasOwn(details, "reason"), "REF_INVALID includes details.reason");
  expect(record, details.ref === "bad:ref", "REF_INVALID preserves ref in error details");
}

function createRegisteredRealMarkdownProject(name: string) {
  const project = createProject(name);
  const markdown = createRealMarkdownAdapter(project);
  writeRegistry(project, [markdown]);
  return project;
}

async function readFirstOutlineRef(project: SmokeProject) {
  const { record, json } = await runReadableJsonCase(project, "CORE-LINK-001 outline real markdown readable-json", [
    "outline",
    project.normalRelPath,
    "--output",
    "readable-json"
  ], "readableOutline");
  const entries = expectObjectArray(record, json.entries, "outline entries are objects");
  expect(record, entries.length > 0, "outline returns entries");
  const ref = expectString(record, entries[0]?.ref, "outline exposes a ref");
  expect(record, ref.length > 0, "outline exposes a nonempty ref");
  return ref;
}

async function readFirstFindRef(project: SmokeProject) {
  const { record, json } = await runReadableJsonCase(project, "CORE-LINK-001 find real markdown readable-json", [
    "find",
    project.normalRelPath,
    "--query",
    "Install",
    "--output",
    "readable-json"
  ], "readableFind");
  const matches = expectObjectArray(record, json.matches, "find matches are objects");
  expect(record, matches.length > 0, "find returns matches");
  const ref = expectString(record, matches[0]?.ref, "find match exposes a ref");
  expect(record, ref.length > 0, "find match exposes a nonempty ref");
  return ref;
}

async function assertInfoReadableOutput(project: SmokeProject) {
  const { record, json } = await runReadableJsonCase(project, "CORE-LINK-001 info real markdown readable-json", [
    "info",
    project.normalRelPath,
    "--output",
    "readable-json"
  ], "readableInfo");
  const display = expectString(record, json.display, "info display is a string");
  const capabilities = expectStringArray(record, json.capabilities, "info capabilities are strings");
  expect(record, display.includes("Markdown | text/markdown"), "info readable result has Markdown display");
  for (const capability of ["outline", "read", "find", "info"]) {
    expect(record, capabilities.includes(capability), `info readable includes ${capability} capability`);
  }
}
