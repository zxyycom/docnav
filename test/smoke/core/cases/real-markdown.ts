import { createProject, createRealMarkdownAdapter, writeRegistry } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectNoJsonPayloadInStderr,
  expectNoProtocolEnvelope,
  expectProtocolFailure,
  expectStderrEmpty,
  parseJson
} from "../assertions.ts";

export function createRealMarkdownLinkTasks() {
  return [
    {
      id: "CORE-LINK-001",
      label: "CORE-LINK-001 real markdown ref handoff chain",
      run: testRealMarkdownRefHandoffChain
    }
  ];
}

export function createRealMarkdownRefErrorTasks() {
  return [
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
  expect(outline, Array.isArray(outlineJson.entries) && outlineJson.entries.length > 0, "outline returns entries");
  const outlineRef = outlineJson.entries[0].ref;
  expect(outline, typeof outlineRef === "string" && outlineRef.length > 0, "outline exposes a nonempty ref");

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
  expect(outlineRead, outlineReadJson.ref === outlineRef, "read preserves outline ref");
  expect(outlineRead, outlineReadJson.content.includes("# Guide"), "read content includes Markdown heading");
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
  expect(find, Array.isArray(findJson.matches) && findJson.matches.length > 0, "find returns matches");
  const findRef = findJson.matches[0].ref;
  expect(find, typeof findRef === "string" && findRef.length > 0, "find match exposes a nonempty ref");

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
  expect(findRead, findReadJson.ref === findRef, "read preserves find ref");
  expect(findRead, findReadJson.content.includes("## Install"), "read content includes Install heading");

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
  expect(info, infoJson.display.includes("Markdown | text/markdown"), "info readable result has Markdown display");
  for (const capability of ["outline", "read", "find", "info"]) {
    expect(info, infoJson.capabilities.includes(capability), `info readable includes ${capability} capability`);
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
  expect(record, Object.hasOwn(json.error.details, "ref"), "REF_INVALID includes details.ref");
  expect(record, Object.hasOwn(json.error.details, "reason"), "REF_INVALID includes details.reason");
  expect(record, json.error.details.ref === "bad:ref", "REF_INVALID preserves ref in error details");
}
