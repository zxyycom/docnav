import { createProject, createRealMarkdownAdapter, writeRegistry } from "../fixtures.ts";
import type { SmokeProject } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectObjectArray,
  expectProtocolSuccess,
  expectReadableViewBlockRestoresField,
  expectStderrEmpty,
  expectString,
  expectStdoutIncludes,
  parseJson,
  parseReadableViewHeader
} from "../assertions.ts";
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
  const markdown = createRealMarkdownAdapter(project);
  writeRegistry(project, [markdown]);

  const readable = await readDocumentReadableJson(project, await readFirstOutlineRef(project));
  await assertReadableViewDocumentOutput(project, readable);
  await assertDefaultDocumentOutput(project, readable.ref);
  await assertProtocolJsonMatchesReadableOutput(project, readable);
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
