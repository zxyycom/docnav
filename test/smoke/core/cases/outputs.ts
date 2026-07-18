import fs from "node:fs";
import path from "node:path";

import { createProject, type SmokeProject } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import { assertConfiguredProtocolEarlyFailure } from "./output-config.ts";
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
    // @case BB-CORE-OUTPUT-001
    {
      id: "CORE-OUTPUT-001",
      label: "CORE-OUTPUT-001 document output boundary",
      run: testDocumentOutputBoundary
    }
  ];
}

async function testDocumentOutputBoundary() {
  const project = createProject("output-boundary");

  const protocol = await readDocumentProtocolJson(project, await readFirstOutlineRef(project));
  const readableViewText = await assertReadableViewDocumentOutput(project, protocol);
  await assertDefaultDocumentOutput(project, protocol.ref, readableViewText);
  await assertEarlyDocumentFailureOutputModes(project);
  await assertRemovedReadableJsonCliRejected(project);
  await assertUnstructuredOutlineOutputModes();
  await assertCostThresholdUnstructuredOutline();
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
  const protocol = await runCli("CORE-OUTPUT-001 early failure protocol-json", [
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

  const readable = await runCli("CORE-OUTPUT-001 early failure readable-view", [
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
  const failure = await runCli("CORE-OUTPUT-001 removed readable-json CLI value is rejected", [
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

async function assertUnstructuredOutlineOutputModes() {
  const content = "raw note\nsecond line";
  const project = createProject("output-boundary-unstructured-outline", {
    config: {
      outline: {
        mode_rules: [
          { path: "docs/raw\\.md", mode: "unstructured_full" }
        ]
      }
    }
  });
  const rawRelPath = "docs/raw.md";
  fs.writeFileSync(path.join(project.root, rawRelPath), content, "utf8");

  const protocol = await runCli("CORE-OUTPUT-001 outline unstructured protocol-json", [
    "outline",
    rawRelPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(protocol, 0);
  expectStderrEmpty(protocol);
  const protocolJson = parseJson(protocol);
  validateSchema(protocol, "protocolResponse", protocolJson);
  expectProtocolSuccess(protocol, protocolJson, "outline");
  const protocolResult = expectJsonObject(protocol, protocolJson.result, "protocol unstructured result is an object");
  expect(protocol, protocolResult.kind === "unstructured", "protocol-json outline uses unstructured kind");
  expect(protocol, protocolResult.reason === "path_rule", "protocol-json outline preserves path_rule reason");
  expect(protocol, protocolResult.content === content, "protocol-json outline contains full content");
  expect(protocol, protocolResult.content_type === "text/markdown", "protocol-json outline preserves Markdown content_type");
  const protocolCost = expectJsonObject(protocol, protocolResult.cost, "protocol unstructured cost is an object");
  const protocolMeasurements = expectObjectArray(
    protocol,
    protocolCost.measurements,
    "protocol unstructured cost measurements are objects"
  );
  expect(protocol, protocolMeasurements.length > 0, "protocol unstructured cost facts are non-empty for Markdown hook");
  expect(protocol, !Object.hasOwn(protocolResult, "display"), "protocol-json unstructured outline omits readable display");
  expect(protocol, !Object.hasOwn(protocolResult, "entries"), "protocol-json unstructured outline omits entries");
  expect(protocol, !Object.hasOwn(protocolResult, "ref"), "protocol-json unstructured outline omits ref");
  expect(protocol, !Object.hasOwn(protocolResult, "page"), "protocol-json unstructured outline omits page");
  expect(protocol, !Object.hasOwn(protocolResult, "continuation"), "protocol-json unstructured outline omits continuation");

  const readableView = await runCli("CORE-OUTPUT-001 outline unstructured readable-view", [
    "outline",
    rawRelPath,
    "--output",
    "readable-view"
  ], { project });
  expectExit(readableView, 0);
  expectStderrEmpty(readableView);
  const header = parseReadableViewHeader(readableView);
  expectNoProtocolEnvelope(readableView, header);
  expect(readableView, header.kind === "unstructured", "readable-view outline uses unstructured kind");
  expect(readableView, header.reason === "path_rule", "readable-view outline preserves path_rule reason");
  expect(readableView, header.content_type === protocolResult.content_type, "readable-view preserves raw content_type");
  const readableCost = expectJsonObject(readableView, header.cost, "readable-view unstructured cost is an object");
  const readableMeasurements = expectObjectArray(
    readableView,
    readableCost.measurements,
    "readable-view unstructured cost measurements are objects"
  );
  expect(
    readableView,
    readableMeasurements.length === protocolMeasurements.length,
    "readable-view derives cost facts from the protocol result"
  );
  expect(readableView, !Object.hasOwn(header, "entries"), "readable-view unstructured outline omits entries");
  expect(readableView, !Object.hasOwn(header, "ref"), "readable-view unstructured outline omits ref");
  expect(readableView, !Object.hasOwn(header, "page"), "readable-view unstructured outline omits page");
  expect(readableView, !Object.hasOwn(header, "continuation"), "readable-view unstructured outline omits continuation");
  expectReadableViewBlockRestoresField(readableView, readableView.stdout, "/content", content);
}

async function assertCostThresholdUnstructuredOutline() {
  const content = "small operational note";
  const project = createProject("output-boundary-unstructured-outline-cost", {
    config: {
      outline: {
        auto_full_read: {
          thresholds: [
            { adapter: "docnav-markdown", unit: "tokens", value: 1000 }
          ]
        }
      }
    }
  });
  const rawRelPath = "docs/small.md";
  fs.writeFileSync(path.join(project.root, rawRelPath), content, "utf8");

  const readable = await runCli("CORE-OUTPUT-001 outline unstructured cost-threshold readable-view", [
    "outline",
    rawRelPath,
    "--output",
    "readable-view"
  ], { project });
  expectExit(readable, 0);
  expectStderrEmpty(readable);
  const header = parseReadableViewHeader(readable);
  expectNoProtocolEnvelope(readable, header);
  expect(readable, header.kind === "unstructured", "cost-threshold readable-view outline uses unstructured kind");
  expect(readable, header.reason === "cost_threshold", "cost-threshold readable-view preserves cost_threshold reason");
  const readableCost = expectJsonObject(readable, header.cost, "cost-threshold readable-view cost is an object");
  const readableMeasurements = expectObjectArray(
    readable,
    readableCost.measurements,
    "cost-threshold readable-view cost measurements are objects"
  );
  expect(readable, readableMeasurements.length > 0, "cost-threshold readable-view cost facts are non-empty");
  expect(readable, !Object.hasOwn(header, "entries"), "cost-threshold readable-view unstructured outline omits entries");
  expect(readable, !Object.hasOwn(header, "page"), "cost-threshold readable-view unstructured outline omits page");
  expectReadableViewBlockRestoresField(readable, readable.stdout, "/content", content);
}
