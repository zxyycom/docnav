import { createProject, createRealMarkdownAdapter, writeRegistry } from "../fixtures.mjs";
import { runCli } from "../harness.mjs";
import {
  expect,
  expectExit,
  expectNoJsonPayloadInStderr,
  expectNoProtocolEnvelope,
  expectProtocolFailure,
  expectStderrEmpty,
  parseJson
} from "../assertions.mjs";
import { validateSchema } from "../harness.mjs";

export function testRealMarkdownOutlineRefRead() {
  const project = createProject("real-markdown-outline-read");
  const markdown = createRealMarkdownAdapter(project);
  writeRegistry(project, [markdown]);

  const outline = runCli("core outline real markdown readable-json", [
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
  const ref = outlineJson.entries[0].ref;
  expect(outline, typeof ref === "string" && ref.length > 0, "outline exposes a nonempty ref");

  const read = runCli("core read real markdown readable-json", [
    "read",
    project.normalRelPath,
    "--ref",
    ref,
    "--output",
    "readable-json"
  ], { project });
  expectExit(read, 0);
  expectStderrEmpty(read);
  const readJson = parseJson(read);
  validateSchema(read, "readableRead", readJson);
  expectNoProtocolEnvelope(read, readJson);
  expect(read, readJson.ref === ref, "read preserves adapter ref");
  expect(read, readJson.content.includes("# Guide"), "read content includes Markdown heading");
  expect(read, readJson.content.includes("target text"), "read content includes fixture body");
  expect(read, readJson.content_type === "text/markdown", "read preserves content_type");
}

export function testRealMarkdownFindRefRead() {
  // 7.11: find → ref → read shared call chain.
  // Core obtains a find match ref, submits it unchanged to read, and the
  // adapter returns content. This proves find refs are usable in read
  // without core parsing Markdown grammar.
  const project = createProject("real-markdown-find-ref-read");
  const markdown = createRealMarkdownAdapter(project);
  writeRegistry(project, [markdown]);

  const find = runCli("core find real markdown readable-json", [
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
  const matchRef = findJson.matches[0].ref;
  expect(find, typeof matchRef === "string" && matchRef.length > 0, "find match exposes a nonempty ref");

  const read = runCli("core read from find ref readable-json", [
    "read",
    project.normalRelPath,
    "--ref",
    matchRef,
    "--output",
    "readable-json"
  ], { project });
  expectExit(read, 0);
  expectStderrEmpty(read);
  const readJson = parseJson(read);
  validateSchema(read, "readableRead", readJson);
  expectNoProtocolEnvelope(read, readJson);
  expect(read, readJson.ref === matchRef, "read preserves find ref");
  expect(read, readJson.content.includes("## Install"), "read content includes Install heading");
  expect(read, readJson.content_type === "text/markdown", "read preserves content_type");
}

export function testRealMarkdownRefInvalid() {
  // 7.9: core CLI passes an invalid ref unchanged to the adapter and maps
  // REF_INVALID. The core layer does not interpret the ref grammar; the
  // adapter owns ref interpretation and error classification.
  const project = createProject("real-markdown-ref-invalid");
  const markdown = createRealMarkdownAdapter(project);
  writeRegistry(project, [markdown]);

  const nonCanonicalRefs = [
    { label: "old heading format", ref: "L4:Guide > Install" },
    { label: "unrecognized grammar", ref: "bad:ref" }
  ];

  for (const item of nonCanonicalRefs) {
    const readableRecord = runCli(`core ref_invalid readable-json (${item.label})`, [
      "read",
      project.normalRelPath,
      "--ref",
      item.ref,
      "--output",
      "readable-json"
    ], { project });
    expectExit(readableRecord, 3);
    const readableJson = parseJson(readableRecord);
    validateSchema(readableRecord, "readableError", readableJson);
    expectNoProtocolEnvelope(readableRecord, readableJson);
    expect(readableRecord, readableJson.code === "REF_INVALID", `core readable REF_INVALID for ${item.label}`);
    expect(readableRecord, Object.hasOwn(readableJson.details, "ref"), `core readable includes details.ref for ${item.label}`);
    expect(readableRecord, Object.hasOwn(readableJson.details, "reason"), `core readable includes details.reason for ${item.label}`);
  }

  // Protocol-json path: verify envelope maps REF_INVALID.
  const protoRecord = runCli("core ref_invalid protocol-json", [
    "read",
    project.normalRelPath,
    "--ref",
    "bad:ref",
    "--output",
    "protocol-json"
  ], { project });
  expectExit(protoRecord, 3);
  expectNoJsonPayloadInStderr(protoRecord);
  const protoJson = parseJson(protoRecord);
  validateSchema(protoRecord, "protocolResponse", protoJson);
  expectProtocolFailure(protoRecord, protoJson, "read", "REF_INVALID");
  expect(protoRecord, Object.hasOwn(protoJson.error.details, "ref"), "core protocol REF_INVALID includes details.ref");
  expect(protoRecord, Object.hasOwn(protoJson.error.details, "reason"), "core protocol REF_INVALID includes details.reason");
  expect(protoRecord, protoJson.error.details.ref === "bad:ref", "core protocol preserves ref in error details");
}

export function testRealMarkdownRefNotFound() {
  // 7.11: canonical grammar but no match returns REF_NOT_FOUND via core.
  // The core just passes the ref; the adapter performs the actual lookup.
  const project = createProject("real-markdown-ref-not-found");
  const markdown = createRealMarkdownAdapter(project);
  writeRegistry(project, [markdown]);

  const record = runCli("core ref_not_found readable-json", [
    "read",
    project.normalRelPath,
    "--ref",
    "H:L999:H1:I1",
    "--output",
    "readable-json"
  ], { project });
  expectExit(record, 3);
  const json = parseJson(record);
  validateSchema(record, "readableError", json);
  expectNoProtocolEnvelope(record, json);
  expect(record, json.code === "REF_NOT_FOUND", "core returns REF_NOT_FOUND for canonical-but-missing ref");
  expect(record, json.details.ref === "H:L999:H1:I1", "core preserves ref in REF_NOT_FOUND details");
}
