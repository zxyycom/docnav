import { createProject, createRealMarkdownAdapter, writeRegistry } from "../fixtures.ts";
import type { SmokeProject } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectNoProtocolEnvelope,
  expectObjectArray,
  expectProtocolSuccess,
  expectReadableViewBlockRestoresField,
  expectStderrEmpty,
  expectString,
  expectStdoutIncludes,
  expectStructuredWarning,
  parseJson,
  parseReadableViewHeader
} from "../assertions.ts";
import type { JsonRecord } from "../assertions.ts";

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
  const markdown = createRealMarkdownAdapter(project);
  writeRegistry(project, [markdown]);

  const readable = await runReadable(project, "CORE-OUTPUT-001 read readable-json output", [
    "read",
    project.normalRelPath,
    "--ref",
    "H:L1:H1:I1",
    "--output",
    "readable-json"
  ], "readableRead");
  const readableRef = expectString(readable.record, readable.json.ref, "read readable-json ref is a string");
  const readableContent = expectString(readable.record, readable.json.content, "read readable-json content is a string");
  expect(readable.record, readable.json.content_type === "text/markdown", "read readable-json preserves content_type");

  const readableView = await runCli("CORE-OUTPUT-001 read readable-view output", [
    "read",
    project.normalRelPath,
    "--ref",
    readableRef,
    "--output",
    "readable-view"
  ], { project });
  expectExit(readableView, 0);
  expectStderrEmpty(readableView);
  expect(readableView, readableView.stdout.trimStart().startsWith("{"), "readable-view stdout starts with JSON header");
  expect(readableView, !readableView.stdout.includes("\"protocol_version\""), "readable-view omits protocol envelope");
  expectStdoutIncludes(readableView, "\"$block\": \"/content\"");
  expectReadableViewBlockRestoresField(readableView, readableView.stdout, "/content", readableContent);

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

  const protocol = await runCli("CORE-OUTPUT-001 read protocol-json output", [
    "read",
    project.normalRelPath,
    "--ref",
    readableRef,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(protocol, 0);
  expectStderrEmpty(protocol);
  const protocolJson = parseJson(protocol);
  validateSchema(protocol, "protocolResponse", protocolJson);
  expectProtocolSuccess(protocol, protocolJson, "read");
  const protocolResult = expectJsonObject(protocol, protocolJson.result, "protocol result is an object");
  expect(protocol, protocolResult.ref === readableRef, "protocol-json result preserves ref");
  expect(protocol, protocolResult.content === readableContent, "protocol-json result matches readable-json content");

  const warningRecord = await runCli("CORE-OUTPUT-001 readable-view warning stays on stdout", [
    "outline",
    project.normalRelPath,
    "--future",
    "--output",
    "readable-view"
  ], { project });
  expectExit(warningRecord, 0);
  expectStderrEmpty(warningRecord);
  const warningHeader = parseReadableViewHeader(warningRecord);
  validateSchema(warningRecord, "readableOutline", warningHeader);
  const warnings = expectObjectArray(warningRecord, warningHeader.warnings, "readable-view warnings are objects");
  expectStructuredWarning(warningRecord, warnings[0], ["--future"], "future flag");
}

async function runReadable(project: SmokeProject, name: string, args: string[], schemaName: string): Promise<{
  json: JsonRecord;
  record: Awaited<ReturnType<typeof runCli>>;
}> {
  const record = await runCli(name, args, { project });
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, schemaName, json);
  expectNoProtocolEnvelope(record, json);
  return { record, json };
}
