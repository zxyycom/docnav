import { createProject, createRealMarkdownAdapter, writeRegistry } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectNoJsonPayloadInStderr,
  expectNoProtocolEnvelope,
  expectObjectArray,
  expectProtocolFailure,
  expectStderrEmpty,
  expectString,
  expectStringArray,
  parseJson
} from "../assertions.ts";

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
  const project = createProject("real-markdown-ref-handoff");
  const markdown = createRealMarkdownAdapter(project);
  writeRegistry(project, [markdown]);

  const outline = await runCli("CORE-LINK-001 outline real markdown readable-json", [
    "outline",
    project.normalRelPath,
    "--output",
    "readable-json"
  ], { project });
  expectExit(outline, 0);
  expectStderrEmpty(outline);
  const outlineJson = parseJson(outline);
  validateSchema(outline, "readableOutline", outlineJson);
  expectNoProtocolEnvelope(outline, outlineJson);
  const outlineEntries = expectObjectArray(outline, outlineJson.entries, "outline entries are objects");
  expect(outline, outlineEntries.length > 0, "outline returns entries");
  const outlineRef = expectString(outline, outlineEntries[0]?.ref, "outline exposes a ref");
  expect(outline, outlineRef.length > 0, "outline exposes a nonempty ref");

  const outlineRead = await runCli("CORE-LINK-001 read outline ref readable-json", [
    "read",
    project.normalRelPath,
    "--ref",
    outlineRef,
    "--output",
    "readable-json"
  ], { project });
  expectExit(outlineRead, 0);
  expectStderrEmpty(outlineRead);
  const outlineReadJson = parseJson(outlineRead);
  validateSchema(outlineRead, "readableRead", outlineReadJson);
  expectNoProtocolEnvelope(outlineRead, outlineReadJson);
  const outlineReadContent = expectString(outlineRead, outlineReadJson.content, "outline read content is a string");
  expect(outlineRead, outlineReadJson.ref === outlineRef, "read preserves outline ref");
  expect(outlineRead, outlineReadContent.includes("# Guide"), "read content includes Markdown heading");
  expect(outlineRead, outlineReadJson.content_type === "text/markdown", "read preserves content_type");

  const find = await runCli("CORE-LINK-001 find real markdown readable-json", [
    "find",
    project.normalRelPath,
    "--query",
    "Install",
    "--output",
    "readable-json"
  ], { project });
  expectExit(find, 0);
  expectStderrEmpty(find);
  const findJson = parseJson(find);
  validateSchema(find, "readableFind", findJson);
  expectNoProtocolEnvelope(find, findJson);
  const matches = expectObjectArray(find, findJson.matches, "find matches are objects");
  expect(find, matches.length > 0, "find returns matches");
  const findRef = expectString(find, matches[0]?.ref, "find match exposes a ref");
  expect(find, findRef.length > 0, "find match exposes a nonempty ref");

  const findRead = await runCli("CORE-LINK-001 read find ref readable-json", [
    "read",
    project.normalRelPath,
    "--ref",
    findRef,
    "--output",
    "readable-json"
  ], { project });
  expectExit(findRead, 0);
  expectStderrEmpty(findRead);
  const findReadJson = parseJson(findRead);
  validateSchema(findRead, "readableRead", findReadJson);
  expectNoProtocolEnvelope(findRead, findReadJson);
  const findReadContent = expectString(findRead, findReadJson.content, "find read content is a string");
  expect(findRead, findReadJson.ref === findRef, "read preserves find ref");
  expect(findRead, findReadContent.includes("## Install"), "read content includes Install heading");

  const info = await runCli("CORE-LINK-001 info real markdown readable-json", [
    "info",
    project.normalRelPath,
    "--output",
    "readable-json"
  ], { project });
  expectExit(info, 0);
  expectStderrEmpty(info);
  const infoJson = parseJson(info);
  validateSchema(info, "readableInfo", infoJson);
  expectNoProtocolEnvelope(info, infoJson);
  const display = expectString(info, infoJson.display, "info display is a string");
  const capabilities = expectStringArray(info, infoJson.capabilities, "info capabilities are strings");
  expect(info, display.includes("Markdown | text/markdown"), "info readable result has Markdown display");
  for (const capability of ["outline", "read", "find", "info"]) {
    expect(info, capabilities.includes(capability), `info readable includes ${capability} capability`);
  }
}

async function testRealMarkdownRefInvalidProtocol() {
  const project = createProject("real-markdown-ref-invalid-protocol");
  const markdown = createRealMarkdownAdapter(project);
  writeRegistry(project, [markdown]);

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
