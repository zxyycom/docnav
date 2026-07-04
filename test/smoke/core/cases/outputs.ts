import fs from "node:fs";
import path from "node:path";

import { createProject } from "../fixtures.ts";
import type { SmokeProject } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectObjectArray,
  expectProtocolSuccess,
  expectReadableFailure,
  expectReadableViewBlockRestoresField,
  expectStderrEmpty,
  expectString,
  expectStdoutIncludes,
  parseJson,
  parseReadableViewHeader
} from "../assertions.ts";
import { exitCodes } from "../config.ts";
import { runReadableJsonCase } from "./readable-output.ts";

interface ReadableDocumentOutput {
  content: string;
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

  const readable = await readDocumentReadableJson(project, await readFirstOutlineRef(project));
  await assertReadableViewDocumentOutput(project, readable);
  await assertDefaultDocumentOutput(project, readable.ref);
  await assertProtocolJsonMatchesReadableOutput(project, readable);
  await assertReadableJsonFailureOutput(project);
  await assertUnstructuredOutlineOutputModes();
  await assertCostThresholdUnstructuredOutline();
}

async function readFirstOutlineRef(project: SmokeProject): Promise<string> {
  const { record, json } = await runReadableJsonCase(project, "CORE-OUTPUT-001 outline readable-json ref source", [
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

async function readDocumentReadableJson(project: SmokeProject, outlineRef: string): Promise<ReadableDocumentOutput> {
  const { record, json } = await runReadableJsonCase(project, "CORE-OUTPUT-001 read readable-json output", [
    "read",
    project.normalRelPath,
    "--ref",
    outlineRef,
    "--output",
    "readable-json"
  ], "readableRead");
  const ref = expectString(record, json.ref, "read readable-json ref is a string");
  const content = expectString(record, json.content, "read readable-json content is a string");
  expect(record, json.content_type === "text/markdown", "read readable-json preserves content_type");
  return { content, ref };
}

async function assertReadableViewDocumentOutput(project: SmokeProject, readable: ReadableDocumentOutput) {
  const readableView = await runCli("CORE-OUTPUT-001 read readable-view output", [
    "read",
    project.normalRelPath,
    "--ref",
    readable.ref,
    "--output",
    "readable-view"
  ], { project });
  expectExit(readableView, 0);
  expectStderrEmpty(readableView);
  expect(readableView, readableView.stdout.trimStart().startsWith("{"), "readable-view stdout starts with JSON header");
  expect(readableView, !readableView.stdout.includes("\"protocol_version\""), "readable-view omits protocol envelope");
  expectStdoutIncludes(readableView, "\"$block\": \"/content\"");
  parseReadableViewHeader(readableView);
  expectReadableViewBlockRestoresField(readableView, readableView.stdout, "/content", readable.content);
}

async function assertDefaultDocumentOutput(project: SmokeProject, readableRef: string) {
  const defaultOutput = await runCli("CORE-OUTPUT-001 read default output is readable-view", [
    "read",
    project.normalRelPath,
    "--ref",
    readableRef
  ], { project });
  expectExit(defaultOutput, 0);
  expectStderrEmpty(defaultOutput);
  expect(defaultOutput, defaultOutput.stdout.trimStart().startsWith("{"), "default output is readable-view JSON header");
  expectStdoutIncludes(defaultOutput, "[block /content bytes=");
  parseReadableViewHeader(defaultOutput);
}

async function assertProtocolJsonMatchesReadableOutput(project: SmokeProject, readable: ReadableDocumentOutput) {
  const protocol = await runCli("CORE-OUTPUT-001 read protocol-json output", [
    "read",
    project.normalRelPath,
    "--ref",
    readable.ref,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(protocol, 0);
  expectStderrEmpty(protocol);
  const protocolJson = parseJson(protocol);
  validateSchema(protocol, "protocolResponse", protocolJson);
  expectProtocolSuccess(protocol, protocolJson, "read");
  const protocolResult = expectJsonObject(protocol, protocolJson.result, "protocol result is an object");
  expect(protocol, protocolResult.ref === readable.ref, "protocol-json result preserves ref");
  expect(protocol, protocolResult.content === readable.content, "protocol-json result matches readable-json content");
}

async function assertReadableJsonFailureOutput(project: SmokeProject) {
  const failure = await runCli("CORE-OUTPUT-001 read readable-json ref failure", [
    "read",
    project.normalRelPath,
    "--ref",
    "bad:ref",
    "--output",
    "readable-json"
  ], { project });
  expectExit(failure, exitCodes.documentRefFormat);
  expectStderrEmpty(failure);
  const json = parseJson(failure);
  validateSchema(failure, "readableError", json);
  const error = expectReadableFailure(failure, json, "REF_INVALID");
  const details = expectJsonObject(failure, error.details, "readable failure details is an object");
  expect(failure, details.ref === "bad:ref", "readable failure preserves ref in details");
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

  const readable = await runReadableJsonCase(project, "CORE-OUTPUT-001 outline unstructured readable-json", [
    "outline",
    rawRelPath,
    "--output",
    "readable-json"
  ], "readableOutline");
  const readableJson = readable.json;
  expect(readable.record, readableJson.kind === "unstructured", "readable-json outline uses unstructured kind");
  expect(readable.record, readableJson.reason === "path_rule", "readable-json outline preserves path_rule reason");
  expect(readable.record, readableJson.content === content, "readable-json outline contains full content");
  expect(readable.record, readableJson.content_type === "text/markdown", "readable-json outline preserves Markdown content_type");
  const readableCost = expectJsonObject(readable.record, readableJson.cost, "readable-json unstructured cost is an object");
  const readableMeasurements = expectObjectArray(
    readable.record,
    readableCost.measurements,
    "readable-json unstructured cost measurements are objects"
  );
  expect(readable.record, readableMeasurements.length > 0, "readable-json unstructured cost facts are non-empty for Markdown hook");
  expect(readable.record, !Object.hasOwn(readableJson, "entries"), "readable-json unstructured outline omits entries");
  expect(readable.record, !Object.hasOwn(readableJson, "ref"), "readable-json unstructured outline omits ref");
  expect(readable.record, !Object.hasOwn(readableJson, "page"), "readable-json unstructured outline omits page");
  expect(readable.record, !Object.hasOwn(readableJson, "continuation"), "readable-json unstructured outline omits continuation");

  const readableView = await runCli("CORE-OUTPUT-001 outline unstructured readable-view", [
    "outline",
    rawRelPath,
    "--output",
    "readable-view"
  ], { project });
  expectExit(readableView, 0);
  expectStderrEmpty(readableView);
  const header = parseReadableViewHeader(readableView);
  expect(readableView, header.kind === "unstructured", "readable-view outline uses unstructured kind");
  expect(readableView, header.reason === "path_rule", "readable-view outline preserves path_rule reason");
  expect(readableView, !Object.hasOwn(header, "entries"), "readable-view unstructured outline omits entries");
  expect(readableView, !Object.hasOwn(header, "page"), "readable-view unstructured outline omits page");
  expectReadableViewBlockRestoresField(readableView, readableView.stdout, "/content", content);

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
  expect(protocol, !Object.hasOwn(protocolResult, "entries"), "protocol-json unstructured outline omits entries");
  expect(protocol, !Object.hasOwn(protocolResult, "ref"), "protocol-json unstructured outline omits ref");
  expect(protocol, !Object.hasOwn(protocolResult, "page"), "protocol-json unstructured outline omits page");
  expect(protocol, !Object.hasOwn(protocolResult, "continuation"), "protocol-json unstructured outline omits continuation");
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

  const readable = await runReadableJsonCase(project, "CORE-OUTPUT-001 outline unstructured cost-threshold readable-json", [
    "outline",
    rawRelPath,
    "--output",
    "readable-json"
  ], "readableOutline");
  const readableJson = readable.json;
  expect(readable.record, readableJson.kind === "unstructured", "cost-threshold readable-json outline uses unstructured kind");
  expect(readable.record, readableJson.reason === "cost_threshold", "cost-threshold readable-json outline preserves cost_threshold reason");
  expect(readable.record, readableJson.content === content, "cost-threshold readable-json outline contains full content");
  const readableCost = expectJsonObject(readable.record, readableJson.cost, "cost-threshold readable-json cost is an object");
  const readableMeasurements = expectObjectArray(
    readable.record,
    readableCost.measurements,
    "cost-threshold readable-json cost measurements are objects"
  );
  expect(readable.record, readableMeasurements.length > 0, "cost-threshold readable-json cost facts are non-empty");
  expect(readable.record, !Object.hasOwn(readableJson, "entries"), "cost-threshold readable-json unstructured outline omits entries");
  expect(readable.record, !Object.hasOwn(readableJson, "page"), "cost-threshold readable-json unstructured outline omits page");
}
