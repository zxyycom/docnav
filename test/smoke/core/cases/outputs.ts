import { createProject, type SmokeProject } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import { assertConfiguredProtocolEarlyFailure } from "./output-config.ts";
import { testUnstructuredOutlineOutputModes } from "./output-unstructured.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectNoProtocolEnvelope,
  expectObjectArray,
  expectProtocolFailure,
  expectProtocolSuccess,
  expectReadableFailure,
  expectReadableViewBlockRestoresField,
  expectStderrEmpty,
  expectString,
  parseJson,
  parseReadableViewHeader
} from "../assertions.ts";
import { exitCodes } from "../config.ts";

interface ProtocolDocumentOutput {
  content: string;
  contentType: string;
  ref: string;
}

export function createDocumentOutputBoundaryTasks() {
  return [
    {
      id: "CORE-OUTPUT-001",
      label: "CORE-OUTPUT-001 document read output modes",
      run: testDocumentReadOutputModes
    },
    {
      id: "CORE-OUTPUT-002",
      label: "CORE-OUTPUT-002 document failure output modes",
      run: testDocumentFailureOutputModes
    },
    {
      id: "CORE-OUTPUT-003",
      label: "CORE-OUTPUT-003 removed output value rejected",
      run: testRemovedOutputValueRejected
    },
    {
      id: "CORE-OUTPUT-004",
      label: "CORE-OUTPUT-004 unstructured outline output modes",
      run: testUnstructuredOutlineOutputModes
    }
  ];
}

async function testDocumentReadOutputModes() {
  const project = createProject("output-boundary");

  const protocol = await readDocumentProtocolJson(project, await readFirstOutlineRef(project));
  const readableViewText = await assertReadableViewDocumentOutput(project, protocol);
  await assertDefaultDocumentOutput(project, protocol.ref, readableViewText);
}

async function testDocumentFailureOutputModes() {
  const project = createProject("output-failure-boundary");
  await assertEarlyDocumentFailureOutputModes(project);
}

async function testRemovedOutputValueRejected() {
  const project = createProject("removed-output-value");
  await assertRemovedReadableJsonCliRejected(project);
}

async function readFirstOutlineRef(project: SmokeProject): Promise<string> {
  const record = await runCli("CORE-OUTPUT-001 outline protocol-json ref source", [
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

async function readDocumentProtocolJson(project: SmokeProject, outlineRef: string): Promise<ProtocolDocumentOutput> {
  const record = await runCli("CORE-OUTPUT-001 read protocol-json output", [
    "read",
    project.normalRelPath,
    "--ref",
    outlineRef,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolSuccess(record, json, "read");
  const result = expectJsonObject(record, json.result, "protocol read result is an object");
  const ref = expectString(record, result.ref, "protocol read ref is a string");
  const content = expectString(record, result.content, "protocol read content is a string");
  const contentType = expectString(record, result.content_type, "protocol read content_type is a string");
  expect(record, contentType === "text/markdown", "protocol read preserves content_type");
  expectJsonObject(record, result.cost, "protocol read preserves raw cost facts");
  expect(record, !Object.hasOwn(result, "display"), "protocol read omits presentation-only display");
  return { content, contentType, ref };
}

async function assertReadableViewDocumentOutput(
  project: SmokeProject,
  protocol: ProtocolDocumentOutput
): Promise<string> {
  const readableView = await runCli("CORE-OUTPUT-001 read readable-view output", [
    "read",
    project.normalRelPath,
    "--ref",
    protocol.ref,
    "--output",
    "readable-view"
  ], { project });
  expectExit(readableView, 0);
  expectStderrEmpty(readableView);
  expect(readableView, readableView.stdout.trimStart().startsWith("{"), "readable-view stdout starts with JSON header");
  const header = parseReadableViewHeader(readableView);
  expectNoProtocolEnvelope(readableView, header);
  expect(readableView, header.ref === protocol.ref, "readable-view preserves protocol ref");
  expect(readableView, header.content_type === protocol.contentType, "readable-view preserves protocol content_type");
  expect(readableView, typeof header.cost === "string", "readable-view derives presentation cost text");
  expectReadableViewBlockRestoresField(readableView, readableView.stdout, "/content", protocol.content);
  return readableView.stdout;
}

async function assertDefaultDocumentOutput(
  project: SmokeProject,
  readableRef: string,
  explicitReadableViewText: string
) {
  const defaultOutput = await runCli("CORE-OUTPUT-001 read default output is readable-view", [
    "read",
    project.normalRelPath,
    "--ref",
    readableRef
  ], { project });
  expectExit(defaultOutput, 0);
  expectStderrEmpty(defaultOutput);
  expect(
    defaultOutput,
    defaultOutput.stdout === explicitReadableViewText,
    "omitted output matches explicit readable-view text"
  );
}

async function assertEarlyDocumentFailureOutputModes(project: SmokeProject) {
  const missingPath = "docs/missing-output-boundary.md";
  const protocol = await runCli("CORE-OUTPUT-002 early failure protocol-json", [
    "read",
    missingPath,
    "--ref",
    "H:L1:H1",
    "--output",
    "protocol-json"
  ], { project });
  expectExit(protocol, exitCodes.documentRefFormat);
  expectStderrEmpty(protocol);
  const protocolJson = parseJson(protocol);
  validateSchema(protocol, "protocolResponse", protocolJson);
  const protocolError = expectProtocolFailure(protocol, protocolJson, "read", "DOCUMENT_NOT_FOUND");
  const protocolDetails = expectJsonObject(protocol, protocolError.details, "protocol early failure details is an object");
  const protocolMessage = expectString(protocol, protocolError.message, "protocol early failure message is a string");
  expect(protocol, !protocol.stdout.includes("[block "), "protocol early failure has no readable block framing");

  const readable = await runCli("CORE-OUTPUT-002 early failure readable-view", [
    "read",
    missingPath,
    "--ref",
    "H:L1:H1",
    "--output",
    "readable-view"
  ], { project });
  expectExit(readable, exitCodes.documentRefFormat);
  expectStderrEmpty(readable);
  const readableHeader = parseReadableViewHeader(readable);
  const readableError = expectReadableFailure(readable, readableHeader, "DOCUMENT_NOT_FOUND");
  const readableDetails = expectJsonObject(readable, readableError.details, "readable early failure details is an object");
  expect(
    readable,
    readableDetails.path === protocolDetails.path,
    "early failure preserves the same path fact across protocol and readable output"
  );
  expectReadableViewBlockRestoresField(readable, readable.stdout, "/error", protocolMessage);
  await assertConfiguredProtocolEarlyFailure(missingPath);
}

async function assertRemovedReadableJsonCliRejected(project: SmokeProject) {
  const failure = await runCli("CORE-OUTPUT-003 removed readable-json CLI value is rejected", [
    "outline",
    project.normalRelPath,
    "--output",
    "readable-json"
  ], { project });
  expectExit(failure, exitCodes.input);
  expectStderrEmpty(failure);
  const header = parseReadableViewHeader(failure);
  const error = expectReadableFailure(failure, header, "INVALID_REQUEST");
  const details = expectJsonObject(failure, error.details, "removed output diagnostic details is an object");
  const reason = expectString(failure, details.reason, "removed output diagnostic reason is a string");
  expect(failure, details.field === "--output", "removed output diagnostic reports --output");
  expect(
    failure,
    reason === "invalid --output: accepted values: readable-view, protocol-json",
    "removed output diagnostic reports the two accepted values"
  );
  expectReadableViewBlockRestoresField(failure, failure.stdout, "/error", reason);
}
