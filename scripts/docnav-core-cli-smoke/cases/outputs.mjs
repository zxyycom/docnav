import { createProject, createRealMarkdownAdapter, writeRegistry } from "../fixtures.mjs";
import { runCli } from "../runner.mjs";
import {
  expect,
  expectExit,
  expectFindResultsEquivalent,
  expectInfoResultsEquivalent,
  expectNoProtocolEnvelope,
  expectOutlineResultsEquivalent,
  expectProtocolSuccess,
  expectReadResultsEquivalent,
  expectStderrEmpty,
  expectStdoutIncludes,
  looksLikeJson,
  parseJson
} from "../assertions.mjs";
import { validateSchema } from "../schemas.mjs";

export function testDocumentOutputMatrix() {
  const project = createProject("output-matrix");
  const markdown = createRealMarkdownAdapter(project);
  writeRegistry(project, [markdown]);

  const readable = readReadableResults(project);
  const ref = readable.outline.entries[0].ref;

  const textCases = [
    {
      name: "outline text output",
      args: ["outline", project.normalRelPath, "--output", "text"],
      checks: [(record) => expectStdoutIncludes(record, ref), (record) => expectStdoutIncludes(record, "page:")]
    },
    {
      name: "read text output",
      args: ["read", project.normalRelPath, "--ref", ref, "--output", "text"],
      checks: [
        (record) => expectStdoutIncludes(record, `ref: ${ref}`),
        (record) => expectStdoutIncludes(record, "content_type: text/markdown"),
        (record) => expectStdoutIncludes(record, "page:")
      ]
    },
    {
      name: "find text output",
      args: ["find", project.normalRelPath, "--query", "target", "--output", "text"],
      checks: [(record) => expectStdoutIncludes(record, "target"), (record) => expectStdoutIncludes(record, "page:")]
    },
    {
      name: "info text output",
      args: ["info", project.normalRelPath, "--output", "text"],
      checks: [(record) => expectStdoutIncludes(record, "Markdown"), (record) => expectStdoutIncludes(record, "capabilities:")]
    }
  ];

  for (const item of textCases) {
    const record = runCli(item.name, item.args, { project });
    expectExit(record, 0);
    expectStderrEmpty(record);
    expect(record, !looksLikeJson(record.stdout), "text stdout is not JSON");
    expect(record, !record.stdout.includes("\"protocol_version\""), "text stdout omits protocol envelope");
    for (const check of item.checks) {
      check(record);
    }
  }

  const protocolCases = [
    {
      name: "outline protocol-json output",
      args: ["outline", project.normalRelPath, "--output", "protocol-json"],
      operation: "outline",
      compare: (record, json) =>
        expectOutlineResultsEquivalent(record, json.result, readable.outline, "outline protocol-json result matches readable-json")
    },
    {
      name: "read protocol-json output",
      args: ["read", project.normalRelPath, "--ref", ref, "--output", "protocol-json"],
      operation: "read",
      compare: (record, json) =>
        expectReadResultsEquivalent(record, json.result, readable.read, "read protocol-json result matches readable-json")
    },
    {
      name: "find protocol-json output",
      args: ["find", project.normalRelPath, "--query", "target", "--output", "protocol-json"],
      operation: "find",
      compare: (record, json) =>
        expectFindResultsEquivalent(record, json.result, readable.find, "find protocol-json result matches readable-json")
    },
    {
      name: "info protocol-json output",
      args: ["info", project.normalRelPath, "--output", "protocol-json"],
      operation: "info",
      compare: (record, json) =>
        expectInfoResultsEquivalent(record, json.result, readable.info, "info protocol-json result matches readable-json")
    }
  ];

  for (const item of protocolCases) {
    const record = runCli(item.name, item.args, { project });
    expectExit(record, 0);
    expectStderrEmpty(record);
    const json = parseJson(record);
    validateSchema(record, "protocolResponse", json);
    expectProtocolSuccess(record, json, item.operation);
    item.compare(record, json);
  }
}

function readReadableResults(project) {
  const outline = runReadable(project, "outline readable-json output", [
    "outline",
    project.normalRelPath,
    "--output",
    "readable-json"
  ], "readableOutline");
  expect(outline.record, Array.isArray(outline.json.entries) && outline.json.entries.length > 0, "outline has entries");
  const ref = outline.json.entries[0].ref;

  const read = runReadable(project, "read readable-json output", [
    "read",
    project.normalRelPath,
    "--ref",
    ref,
    "--output",
    "readable-json"
  ], "readableRead");
  expect(read.record, read.json.content_type === "text/markdown", "read readable-json preserves content_type");

  const find = runReadable(project, "find readable-json output", [
    "find",
    project.normalRelPath,
    "--query",
    "target",
    "--output",
    "readable-json"
  ], "readableFind");

  const info = runReadable(project, "info readable-json output", [
    "info",
    project.normalRelPath,
    "--output",
    "readable-json"
  ], "readableInfo");

  return {
    outline: outline.json,
    read: read.json,
    find: find.json,
    info: info.json
  };
}

function runReadable(project, name, args, schemaName) {
  const record = runCli(name, args, { project });
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, schemaName, json);
  expectNoProtocolEnvelope(record, json);
  return { record, json };
}

