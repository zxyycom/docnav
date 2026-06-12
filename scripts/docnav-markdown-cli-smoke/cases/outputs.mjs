import { fixture, setNormalRef } from "../fixtures.mjs";
import { runCli, runProtocolResponseCase, runSuccessfulJsonCase } from "../harness.mjs";
import {
  expect,
  expectExit,
  expectFindResultsEquivalent,
  expectIncludes,
  expectInfoResultsEquivalent,
  expectNoProtocolEnvelope,
  expectNormalFindResult,
  expectOutlineResultsEquivalent,
  expectReadResultsEquivalent,
  expectStderrEmpty,
  expectStdoutIncludes,
  looksLikeJson
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
      textChecks: [
        (record) => expectStdoutIncludes(record, ref),
        (record) => expectStdoutIncludes(record, "Guide"),
        (record) => expectStdoutIncludes(record, "H1"),
        (record) => expectStdoutIncludes(record, "page:")
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
      textChecks: [
        (record) => expectStdoutIncludes(record, `ref: ${ref}`),
        (record) => expectStdoutIncludes(record, "# Guide"),
        (record) => expectStdoutIncludes(record, "target text"),
        (record) => expectStdoutIncludes(record, "content_type: text/markdown"),
        (record) => expectStdoutIncludes(record, "cost: "),
        (record) => expectStdoutIncludes(record, "page:")
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
      textChecks: [
        (record) => expect(record, /^L\d+(?:#\d+)?:/m.test(record.stdout), "find text includes canonical ref"),
        (record) => expectStdoutIncludes(record, "target text"),
        (record) => expectStdoutIncludes(record, "target result"),
        (record) =>
          expect(record, (record.stdout.match(/target/g) ?? []).length >= 2, "find text includes both target matches"),
        (record) => expectStdoutIncludes(record, "page:")
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
      textChecks: [
        (record) => expectStdoutIncludes(record, "Markdown"),
        (record) => expectStdoutIncludes(record, "text/markdown"),
        (record) => expectStdoutIncludes(record, "capabilities:"),
        (record) => expectStdoutIncludes(record, "outline"),
        (record) => expectStdoutIncludes(record, "read"),
        (record) => expectStdoutIncludes(record, "find"),
        (record) => expectStdoutIncludes(record, "info")
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

  for (const item of operations) {
    const record = runCli(`${item.operation} normal text`, [...item.args, "--output", "text"]);
    expectExit(record, 0);
    expectStderrEmpty(record);
    expect(record, !looksLikeJson(record.stdout), "text stdout is not JSON");
    expect(record, !record.stdout.includes("\"protocol_version\""), "text stdout omits protocol envelope");
    for (const check of item.textChecks) {
      check(record);
    }
  }

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
