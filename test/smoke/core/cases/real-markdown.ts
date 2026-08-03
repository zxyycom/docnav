import { createProject } from "../fixtures.ts";
import type { SmokeProject } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectNoJsonPayloadInStderr,
  expectNoProtocolEnvelope,
  expectObjectArray,
  expectProtocolFailure,
  expectProtocolSuccess,
  expectStderrEmpty,
  expectString,
  parseJson,
  parseReadableViewHeader
} from "../assertions.ts";
import { assertDocumentHeadOutputModes } from "./real-markdown-document-head.ts";
import {
  testMaxHeadingLevelAdapterValidation,
  testMaxHeadingLevelCliOption
} from "./real-markdown-options.ts";

interface ReadableViewRefHandoffExpectation {
  contentIncludes: string;
  contentIncludesSummary: string;
  contentType?: {
    summary: string;
    value: string;
  };
  refSummary: string;
}

export function createRealMarkdownLinkTasks() {
  return [
    {
      id: "CORE-LINK-001",
      label: "CORE-LINK-001 outline ref handoff",
      run: testOutlineRefHandoff
    },
    {
      id: "CORE-LINK-002",
      label: "CORE-LINK-002 find ref handoff",
      run: testFindRefHandoff
    },
    {
      id: "CORE-INFO-001",
      label: "CORE-INFO-001 Markdown info readable output",
      run: testInfoReadableOutput
    },
    {
      id: "CORE-MD-OPTIONS-001",
      label: "CORE-MD-OPTIONS-001 Markdown option success behavior",
      run: testMaxHeadingLevelCliOption
    },
    {
      id: "CORE-MD-OPTIONS-002",
      label: "CORE-MD-OPTIONS-002 Markdown option adapter validation",
      run: testMaxHeadingLevelAdapterValidation
    },
    {
      id: "CORE-MD-DOCHEAD-001",
      label: "CORE-MD-DOCHEAD-001 Markdown document head output modes",
      run: testDocumentHeadOutputModes
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

async function testOutlineRefHandoff() {
  const project = createRegisteredRealMarkdownProject("real-markdown-outline-ref-handoff");
  const outlineRef = await readFirstOutlineRef(project);
  await assertReadableViewReadRefHandoff(
    project,
    "CORE-LINK-001 read outline ref readable-view",
    project.normalRelPath,
    outlineRef,
    {
      contentIncludes: "# Guide",
      contentIncludesSummary: "read content includes Markdown heading",
      contentType: {
        summary: "read preserves content_type",
        value: "text/markdown"
      },
      refSummary: "read preserves outline ref"
    }
  );
}

async function testFindRefHandoff() {
  const project = createRegisteredRealMarkdownProject("real-markdown-find-ref-handoff");
  const findRef = await readFirstFindRef(project);
  await assertReadableViewReadRefHandoff(
    project,
    "CORE-LINK-002 read find ref readable-view",
    project.normalRelPath,
    findRef,
    {
      contentIncludes: "## Install",
      contentIncludesSummary: "read content includes Install heading",
      refSummary: "read preserves find ref"
    }
  );
}

async function testInfoReadableOutput() {
  const project = createRegisteredRealMarkdownProject("real-markdown-info-readable");
  await assertInfoReadableOutput(project);
}

async function testDocumentHeadOutputModes() {
  const project = createRegisteredRealMarkdownProject("real-markdown-document-head");
  await assertDocumentHeadOutputModes(project);
}

async function assertReadableViewReadRefHandoff(
  project: SmokeProject,
  name: string,
  documentPath: string,
  ref: string,
  expectation: ReadableViewRefHandoffExpectation
) {
  const record = await runCli(name, [
    "read",
    documentPath,
    "--adapter",
    "docnav-markdown",
    "--ref",
    ref,
    "--output",
    "readable-view"
  ], { project });
  expectExit(record, 0);
  expectStderrEmpty(record);
  const header = parseReadableViewHeader(record);
  expectNoProtocolEnvelope(record, header);
  expect(record, header.ref === ref, expectation.refSummary);
  expect(record, record.stdout.includes("[block /content bytes="), "readable-view read has a content block");
  expect(
    record,
    record.stdout.includes(expectation.contentIncludes),
    expectation.contentIncludesSummary
  );
  if (expectation.contentType) {
    expect(
      record,
      header.content_type === expectation.contentType.value,
      expectation.contentType.summary
    );
  }
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
  return createProject(name);
}

async function readFirstOutlineRef(project: SmokeProject) {
  const record = await runCli("CORE-LINK-001 outline real markdown protocol-json", [
    "outline",
    project.normalRelPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolSuccess(record, json, "outline");
  const result = expectJsonObject(record, json.result, "outline result is an object");
  const entries = expectObjectArray(record, result.entries, "outline entries are objects");
  expect(record, entries.length > 0, "outline returns entries");
  const ref = expectString(record, entries[0]?.ref, "outline exposes a ref");
  expect(record, ref.length > 0, "outline exposes a nonempty ref");
  return ref;
}

async function readFirstFindRef(project: SmokeProject) {
  const record = await runCli("CORE-LINK-002 find real markdown protocol-json", [
    "find",
    project.normalRelPath,
    "--query",
    "Install",
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolSuccess(record, json, "find");
  const result = expectJsonObject(record, json.result, "find result is an object");
  const matches = expectObjectArray(record, result.matches, "find matches are objects");
  expect(record, matches.length > 0, "find returns matches");
  const ref = expectString(record, matches[0]?.ref, "find match exposes a ref");
  expect(record, ref.length > 0, "find match exposes a nonempty ref");
  return ref;
}

async function assertInfoReadableOutput(project: SmokeProject) {
  const record = await runCli("CORE-INFO-001 info real markdown readable-view", [
    "info",
    project.normalRelPath,
    "--output",
    "readable-view"
  ], { project });
  expectExit(record, 0);
  expectStderrEmpty(record);
  const header = parseReadableViewHeader(record);
  expectNoProtocolEnvelope(record, header);
  const display = expectString(record, header.display, "info display is a string");
  expect(record, display.includes("Markdown | text/markdown"), "info readable result has Markdown display");
}
