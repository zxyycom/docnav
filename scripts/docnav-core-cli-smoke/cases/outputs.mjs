import { createProject, createRealMarkdownAdapter, writeRegistry } from "../fixtures.mjs";
import { runCli, validateSchema } from "../harness.mjs";
import {
  expect,
  expectExit,
  expectNoProtocolEnvelope,
  expectProtocolSuccess,
  expectReadableViewBlockRestoresField,
  expectStderrEmpty,
  expectStdoutIncludes,
  expectStructuredWarning,
  parseJson,
  parseReadableViewHeader
} from "../assertions.mjs";

export function createDocumentOutputBoundaryTasks() {
  return [
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
  expect(readable.record, readable.json.content_type === "text/markdown", "read readable-json preserves content_type");

  const readableView = await runCli("CORE-OUTPUT-001 read readable-view output", [
    "read",
    project.normalRelPath,
    "--ref",
    readable.json.ref,
    "--output",
    "readable-view"
  ], { project });
  expectExit(readableView, 0);
  expectStderrEmpty(readableView);
  expect(readableView, readableView.stdout.trimStart().startsWith("{"), "readable-view stdout starts with JSON header");
  expect(readableView, !readableView.stdout.includes("\"protocol_version\""), "readable-view omits protocol envelope");
  expectStdoutIncludes(readableView, "\"$block\": \"/content\"");
  expectReadableViewBlockRestoresField(readableView, readableView.stdout, "/content", readable.json.content);

  const defaultOutput = await runCli("CORE-OUTPUT-001 read default output is readable-view", [
    "read",
    project.normalRelPath,
    "--ref",
    readable.json.ref
  ], { project });
  expectExit(defaultOutput, 0);
  expectStderrEmpty(defaultOutput);
  expect(defaultOutput, defaultOutput.stdout.trimStart().startsWith("{"), "default output is readable-view JSON header");
  expectStdoutIncludes(defaultOutput, "[block /content bytes=");

  const protocol = await runCli("CORE-OUTPUT-001 read protocol-json output", [
    "read",
    project.normalRelPath,
    "--ref",
    readable.json.ref,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(protocol, 0);
  expectStderrEmpty(protocol);
  const protocolJson = parseJson(protocol);
  validateSchema(protocol, "protocolResponse", protocolJson);
  expectProtocolSuccess(protocol, protocolJson, "read");
  expect(protocol, protocolJson.result.ref === readable.json.ref, "protocol-json result preserves ref");
  expect(protocol, protocolJson.result.content === readable.json.content, "protocol-json result matches readable-json content");

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
  expectStructuredWarning(warningRecord, warningHeader.warnings?.[0], ["--future"], "unknown flag");
}

async function runReadable(project, name, args, schemaName) {
  const record = await runCli(name, args, { project });
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, schemaName, json);
  expectNoProtocolEnvelope(record, json);
  return { record, json };
}
