import { createProject, createRealMarkdownAdapter, writeRegistry } from "../fixtures.mjs";
import { runCli, validateSchema } from "../harness.mjs";
import {
  expect,
  expectExit,
  expectFindResultsEquivalent,
  expectInfoResultsEquivalent,
  expectNoReadableViewBlocks,
  expectNoProtocolEnvelope,
  expectOutlineResultsEquivalent,
  expectProtocolSuccess,
  expectReadableViewBlockRestoresField,
  expectReadResultsEquivalent,
  expectStderrEmpty,
  expectStdoutIncludes,
  expectStructuredWarning,
  parseJson,
  parseReadableViewHeader
} from "../assertions.mjs";

export function createDocumentOutputMatrixTasks() {
  return [{ id: "core-output-matrix", run: testDocumentOutputMatrix }];
}

async function testDocumentOutputMatrix() {
  const project = createProject("output-matrix");
  const markdown = createRealMarkdownAdapter(project);
  writeRegistry(project, [markdown]);

  const readable = await readReadableResults(project);
  const ref = readable.outline.entries[0].ref;

  // ── readable-view (default) output checks ────────────────────────────

  const readableViewChecks = [
    {
      name: "outline readable-view output",
      args: ["outline", project.normalRelPath, "--output", "readable-view"],
      checks: [
        (record) => {
          const header = parseReadableViewHeader(record);
          expectOutlineResultsEquivalent(record, header, readable.outline, "outline readable-view header matches readable-json");
          expectNoReadableViewBlocks(record, record.stdout, "outline readable-view");
        },
        (record) => expectStdoutIncludes(record, ref),
        (record) => expectStdoutIncludes(record, "display")
      ]
    },
    {
      name: "read readable-view output",
      args: ["read", project.normalRelPath, "--ref", ref, "--output", "readable-view"],
      checks: [
        (record) => expectStdoutIncludes(record, "\"$block\": \"/content\""),
        (record) => expectStdoutIncludes(record, "[block /content bytes="),
        (record) => expectStdoutIncludes(record, "[endblock /content]"),
        (record) => expectStdoutIncludes(record, "content_type"),
        (record) => expectStdoutIncludes(record, ref),
        (record) => expectReadableViewBlockRestoresField(record, record.stdout, "/content", readable.read.content)
      ]
    },
    {
      name: "find readable-view output",
      args: ["find", project.normalRelPath, "--query", "target", "--output", "readable-view"],
      checks: [
        (record) => {
          const header = parseReadableViewHeader(record);
          expectFindResultsEquivalent(record, header, readable.find, "find readable-view header matches readable-json");
          expectNoReadableViewBlocks(record, record.stdout, "find readable-view");
        }
      ]
    },
    {
      name: "info readable-view output",
      args: ["info", project.normalRelPath, "--output", "readable-view"],
      checks: [
        (record) => {
          const header = parseReadableViewHeader(record);
          expectInfoResultsEquivalent(record, header, readable.info, "info readable-view header matches readable-json");
          expectNoReadableViewBlocks(record, record.stdout, "info readable-view");
        }
      ]
    }
  ];

  // ── default output (readable-view) without explicit --output ────────

  const defaultCases = [
    {
      name: "outline default output (readable-view)",
      args: ["outline", project.normalRelPath],
      checks: [
        (record) => {
          const header = parseReadableViewHeader(record);
          expectOutlineResultsEquivalent(record, header, readable.outline, "default outline readable-view matches readable-json");
          expectNoReadableViewBlocks(record, record.stdout, "default outline readable-view");
        },
        (record) => expectStdoutIncludes(record, ref)
      ]
    },
    {
      name: "read default output (readable-view)",
      args: ["read", project.normalRelPath, "--ref", ref],
      checks: [
        (record) => expectStdoutIncludes(record, "\"$block\": \"/content\""),
        (record) => expectStdoutIncludes(record, "[block /content bytes="),
        (record) => expectReadableViewBlockRestoresField(record, record.stdout, "/content", readable.read.content)
      ]
    },
    {
      name: "find default output (readable-view)",
      args: ["find", project.normalRelPath, "--query", "target"],
      checks: [
        (record) => {
          const header = parseReadableViewHeader(record);
          expectFindResultsEquivalent(record, header, readable.find, "default find readable-view matches readable-json");
          expectNoReadableViewBlocks(record, record.stdout, "default find readable-view");
        }
      ]
    },
    {
      name: "info default output (readable-view)",
      args: ["info", project.normalRelPath],
      checks: [
        (record) => {
          const header = parseReadableViewHeader(record);
          expectInfoResultsEquivalent(record, header, readable.info, "default info readable-view matches readable-json");
          expectNoReadableViewBlocks(record, record.stdout, "default info readable-view");
        }
      ]
    }
  ];

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

  for (const item of readableViewChecks) {
    const record = await runCli(item.name, item.args, { project });
    expectExit(record, 0);
    expectStderrEmpty(record);
    // readable-view starts with JSON header.
    expect(record, record.stdout.trimStart().startsWith("{"), "readable-view stdout starts with JSON header");
    expect(record, !record.stdout.includes("\"protocol_version\""), "readable-view omits protocol envelope");
    for (const check of item.checks) {
      check(record);
    }
  }

  const warningRecord = await runCli("outline readable-view warning stays on stdout", [
    "outline",
    project.normalRelPath,
    "--future",
    "--output",
    "readable-view"
  ], { project });
  expectExit(warningRecord, 0);
  expectStderrEmpty(warningRecord);
  const warningHeader = parseReadableViewHeader(warningRecord);
  expectOutlineResultsEquivalent(
    warningRecord,
    warningHeader,
    readable.outline,
    "warning readable-view outline fields match readable-json"
  );
  expectStructuredWarning(warningRecord, warningHeader.warnings?.[0], ["--future"], "unknown flag");

  for (const item of defaultCases) {
    const record = await runCli(item.name, item.args, { project });
    expectExit(record, 0);
    expectStderrEmpty(record);
    expect(record, record.stdout.trimStart().startsWith("{"), "default output is readable-view JSON header");
    for (const check of item.checks) {
      check(record);
    }
  }

  for (const item of protocolCases) {
    const record = await runCli(item.name, item.args, { project });
    expectExit(record, 0);
    expectStderrEmpty(record);
    const json = parseJson(record);
    validateSchema(record, "protocolResponse", json);
    expectProtocolSuccess(record, json, item.operation);
    item.compare(record, json);
  }
}

async function readReadableResults(project) {
  const outline = await runReadable(project, "outline readable-json output", [
    "outline",
    project.normalRelPath,
    "--output",
    "readable-json"
  ], "readableOutline");
  expect(outline.record, Array.isArray(outline.json.entries) && outline.json.entries.length > 0, "outline has entries");
  const ref = outline.json.entries[0].ref;

  const read = await runReadable(project, "read readable-json output", [
    "read",
    project.normalRelPath,
    "--ref",
    ref,
    "--output",
    "readable-json"
  ], "readableRead");
  expect(read.record, read.json.content_type === "text/markdown", "read readable-json preserves content_type");

  const find = await runReadable(project, "find readable-json output", [
    "find",
    project.normalRelPath,
    "--query",
    "target",
    "--output",
    "readable-json"
  ], "readableFind");

  const info = await runReadable(project, "info readable-json output", [
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

async function runReadable(project, name, args, schemaName) {
  const record = await runCli(name, args, { project });
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, schemaName, json);
  expectNoProtocolEnvelope(record, json);
  return { record, json };
}
