import { fixture, setNormalRef } from "../fixtures.mjs";
import { runCli, runProtocolResponseCase, runSuccessfulJsonCase } from "../harness.mjs";
import {
  expect,
  expectExit,
  expectFindResultsEquivalent,
  expectIncludes,
  expectInfoResultsEquivalent,
  expectNoReadableViewBlocks,
  expectNoProtocolEnvelope,
  expectNormalFindResult,
  expectOutlineResultsEquivalent,
  expectReadableViewBlockRestoresField,
  expectReadResultsEquivalent,
  expectStderrEmpty,
  parseReadableViewHeader,
} from "../assertions.mjs";

export function testDocumentOutputMatrix() {
  const normal = fixture("normal.md");
  const readable = {};

  const { record: outlineRecord, json: outline } = runSuccessfulJsonCase(
    "outline normal readable-json",
    ["outline", normal, "--output", "readable-json"],
    {
      schema: "readableOutline",
      check: (record, json) => {
        expectNoProtocolEnvelope(record, json);
        expect(record, Array.isArray(json.entries) && json.entries.length > 0, "outline returns entries");
        expect(record, json.page === null, "outline page is null for normal fixture");
        expect(
          record,
          new Set(json.entries.map((entry) => entry.ref)).size === json.entries.length,
          "outline refs are unique"
        );
        expect(record, json.entries[0].display.includes("H1"), "outline first entry identifies a top-level heading");
      }
    }
  );
  readable.outline = outline;
  const ref = outline.entries[0].ref;
  setNormalRef(ref);
  expect(outlineRecord, typeof ref === "string" && ref.length > 0, "outline exposes a nonempty ref");

  const operations = [
    {
      operation: "outline",
      args: ["outline", normal],
      schema: "readableOutline",
      readableViewChecks: [
        (record) => {
          const header = parseReadableViewHeader(record);
          expectOutlineResultsEquivalent(record, header, readable.outline, "outline readable-view header matches readable-json");
          expect(record, record.stdout.trimStart().startsWith("{"), "outline readable-view stdout starts with JSON header");
          expect(record, !record.stdout.includes("\"protocol_version\""), "outline readable-view omits protocol envelope");
          expectNoReadableViewBlocks(record, record.stdout, "outline readable-view");
        }
      ],
      protocolCheck: (record, json) => {
        expectOutlineResultsEquivalent(
          record,
          json.result,
          readable.outline,
          "outline protocol-json result matches readable-json"
        );
      }
    },
    {
      operation: "read",
      args: ["read", normal, "--ref", ref],
      schema: "readableRead",
      readableCheck: (record, json) => {
        expect(record, json.ref === ref, "read result preserves ref");
        expect(record, json.content.includes("# Guide"), "read content includes heading");
        expect(record, json.content.includes("target text"), "read content includes target text");
        expect(record, json.content_type === "text/markdown", "read content_type is text/markdown");
        expect(record, json.page === null, "read page is null for normal fixture");
      },
      readableViewChecks: [
        (record) => {
          const header = parseReadableViewHeader(record);
          expect(record, record.stdout.trimStart().startsWith("{"), "read readable-view stdout starts with JSON header");
          expect(record, !record.stdout.includes("\"protocol_version\""), "read readable-view omits protocol envelope");
          // Header contains block reference, not raw content.
          expect(record, header.content?.$block === "/content", "read header has $block reference");
          expect(
            record,
            header.content?.bytes === Buffer.byteLength(readable.read.content, "utf8"),
            "read header has matching bytes field"
          );
          expect(record, header.content_type === readable.read.content_type, "read header has content_type");
          expect(record, header.ref === readable.read.ref, "read header has ref");
          expect(record, header.cost === readable.read.cost, "read header has cost");
          expect(record, header.page === readable.read.page, "read header has page");
          // Block section present with markers.
          expect(record, record.stdout.includes("[block /content bytes="), "read has [block /content] marker");
          expect(record, record.stdout.includes("[endblock /content]"), "read has [endblock /content] marker");
          // Block payload contains the original content.
          expect(record, record.stdout.includes("# Guide"), "read block content includes heading");
          expect(record, record.stdout.includes("target text"), "read block content includes target text");
          expectReadableViewBlockRestoresField(record, record.stdout, "/content", readable.read.content);
        }
      ],
      protocolCheck: (record, json) => {
        expect(record, json.result.ref === ref, "read protocol result preserves ref");
        expect(record, json.result.content_type === "text/markdown", "read protocol result has content_type");
        expectReadResultsEquivalent(
          record,
          json.result,
          readable.read,
          "read protocol-json result matches readable-json"
        );
      }
    },
    {
      operation: "find",
      args: ["find", normal, "--query", "target"],
      schema: "readableFind",
      readableCheck: (record, json) => expectNormalFindResult(record, json, "readable find"),
      readableViewChecks: [
        (record) => {
          const header = parseReadableViewHeader(record);
          expectFindResultsEquivalent(record, header, readable.find, "find readable-view header matches readable-json");
          expect(record, record.stdout.trimStart().startsWith("{"), "find readable-view stdout starts with JSON header");
          expect(record, !record.stdout.includes("\"protocol_version\""), "find readable-view omits protocol envelope");
          expectNoReadableViewBlocks(record, record.stdout, "find readable-view");
        }
      ],
      protocolCheck: (record, json) => {
        expectNormalFindResult(record, json.result, "protocol find");
        expectFindResultsEquivalent(
          record,
          json.result,
          readable.find,
          "find protocol-json result matches readable-json"
        );
      }
    },
    {
      operation: "info",
      args: ["info", normal],
      schema: "readableInfo",
      readableCheck: (record, json) => {
        expect(record, json.display.includes("Markdown | text/markdown"), "info readable result has Markdown display");
        for (const capability of ["outline", "read", "find", "info"]) {
          expectIncludes(record, json.capabilities, capability, `info readable includes ${capability} capability`);
        }
      },
      readableViewChecks: [
        (record) => {
          const header = parseReadableViewHeader(record);
          expectInfoResultsEquivalent(record, header, readable.info, "info readable-view header matches readable-json");
          expect(record, record.stdout.trimStart().startsWith("{"), "info readable-view stdout starts with JSON header");
          expect(record, !record.stdout.includes("\"protocol_version\""), "info readable-view omits protocol envelope");
          expectNoReadableViewBlocks(record, record.stdout, "info readable-view");
        }
      ],
      protocolCheck: (record, json) => {
        expect(record, json.result.display.includes("Markdown | text/markdown"), "info protocol result has display");
        expectIncludes(record, json.result.capabilities, "read", "info protocol result includes read capability");
        expectInfoResultsEquivalent(
          record,
          json.result,
          readable.info,
          "info protocol-json result matches readable-json"
        );
      }
    }
  ];

  // Collect readable-json results for protocol-json comparison.
  for (const item of operations.slice(1)) {
    const { json } = runSuccessfulJsonCase(
      `${item.operation} normal readable-json`,
      [...item.args, "--output", "readable-json"],
      {
        schema: item.schema,
        check: (record, value) => {
          expectNoProtocolEnvelope(record, value);
          item.readableCheck(record, value);
        }
      }
    );
    readable[item.operation] = json;
  }

  // 3.5: readable-view output tests — replacing the old "text" output mode.
  for (const item of operations) {
    const record = runCli(
      `${item.operation} normal readable-view`,
      [...item.args, "--output", "readable-view"]
    );
    expectExit(record, 0);
    expectStderrEmpty(record);
    expect(record, record.stdout.trimStart().startsWith("{"), "readable-view stdout starts with JSON header");
    expect(record, !record.stdout.includes("\"protocol_version\""), "readable-view stdout omits protocol envelope");
    for (const check of item.readableViewChecks) {
      check(record);
    }
  }

  // 3.5: default output mode (no --output flag) is also readable-view.
  for (const item of operations) {
    const record = runCli(`${item.operation} normal default output`, item.args);
    expectExit(record, 0);
    expectStderrEmpty(record);
    expect(record, record.stdout.trimStart().startsWith("{"), "default output starts with JSON header (readable-view)");
    expect(record, !record.stdout.includes("\"protocol_version\""), "default output omits protocol envelope");
    for (const check of item.readableViewChecks) {
      check(record);
    }
  }

  // 3.5: protocol-json output tests (unchanged boundary).
  for (const item of operations) {
    runProtocolResponseCase(
      `${item.operation} normal protocol-json`,
      [...item.args, "--output", "protocol-json"],
      {
        operation: item.operation,
        check: item.protocolCheck
      }
    );
  }
}
