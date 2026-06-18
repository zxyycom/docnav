import { fixture, setNormalRef } from "../fixtures.ts";
import { runCli, runProtocolResponseCase, runSuccessfulJsonCase } from "../harness.ts";
import {
  expect,
  expectExit,
  expectNoProtocolEnvelope,
  expectReadableViewBlockRestoresField,
  expectReadResultsEquivalent,
  expectStderrEmpty,
  parseReadableViewHeader
} from "../assertions.ts";

export function createDocumentLinkTasks() {
  return [
    {
      id: "MD-LINK-001",
      label: "MD-LINK-001 markdown outline find read info chain",
      run: testMarkdownDocumentLinkChain
    }
  ];
}

export function createDocumentOutputBoundaryTasks() {
  return [
    {
      id: "MD-OUTPUT-001",
      label: "MD-OUTPUT-001 markdown output boundary",
      run: testMarkdownOutputBoundary
    }
  ];
}

async function testMarkdownDocumentLinkChain() {
  const normal = fixture("normal.md");

  const { record: outlineRecord, json: outline } = await runSuccessfulJsonCase(
    "MD-LINK-001 outline normal readable-json",
    ["outline", normal, "--output", "readable-json"],
    {
      schema: "readableOutline",
      check: (record: ExternalValue, json: ExternalValue) => {
        expectNoProtocolEnvelope(record, json);
        expect(record, Array.isArray(json.entries) && json.entries.length > 0, "outline returns entries");
        expect(record, json.page === null, "outline page is null for normal fixture");
        expect(
          record,
          new Set(json.entries.map((entry: ExternalValue) => entry.ref)).size === json.entries.length,
          "outline refs are unique"
        );
        expect(record, json.entries[0].display.includes("H1"), "outline first entry identifies a top-level heading");
      }
    }
  );
  const outlineRef = outline.entries[0].ref;
  setNormalRef(outlineRef);
  expect(outlineRecord, typeof outlineRef === "string" && outlineRef.length > 0, "outline exposes a nonempty ref");

  const { json: read } = await runSuccessfulJsonCase(
    "MD-LINK-001 read outline ref readable-json",
    ["read", normal, "--ref", outlineRef, "--output", "readable-json"],
    {
      schema: "readableRead",
      check: (record: ExternalValue, json: ExternalValue) => {
        expectNoProtocolEnvelope(record, json);
        expect(record, json.ref === outlineRef, "read result preserves outline ref");
        expect(record, json.content.includes("# Guide"), "read content includes heading");
        expect(record, json.content.includes("target text"), "read content includes target text");
        expect(record, json.content_type === "text/markdown", "read content_type is text/markdown");
        expect(record, json.page === null, "read page is null for normal fixture");
      }
    }
  );

  const { json: find } = await runSuccessfulJsonCase(
    "MD-LINK-001 find target readable-json",
    ["find", normal, "--query", "target", "--output", "readable-json"],
    {
      schema: "readableFind",
      check: (record: ExternalValue, json: ExternalValue) => {
        expectNoProtocolEnvelope(record, json);
        expect(record, Array.isArray(json.matches) && json.matches.length > 0, "find returns matches");
        expect(record, typeof json.matches[0].ref === "string" && json.matches[0].ref.length > 0, "find exposes ref");
        expect(record, json.matches[0].display.includes("target"), "find display includes query text");
      }
    }
  );

  await runSuccessfulJsonCase(
    "MD-LINK-001 read find ref readable-json",
    ["read", normal, "--ref", find.matches[0].ref, "--output", "readable-json"],
    {
      schema: "readableRead",
      check: (record: ExternalValue, json: ExternalValue) => {
        expectNoProtocolEnvelope(record, json);
        expect(record, json.ref === find.matches[0].ref, "read preserves find ref");
        expect(record, json.content_type === read.content_type, "read from find ref preserves content_type");
      }
    }
  );

  await runSuccessfulJsonCase("MD-LINK-001 info normal readable-json", ["info", normal, "--output", "readable-json"], {
    schema: "readableInfo",
    check: (record: ExternalValue, json: ExternalValue) => {
      expectNoProtocolEnvelope(record, json);
      expect(record, json.display.includes("Markdown | text/markdown"), "info readable result has Markdown display");
      for (const capability of ["outline", "read", "find", "info"]) {
        expect(record, json.capabilities.includes(capability), `info readable includes ${capability} capability`);
      }
    }
  });
}

async function testMarkdownOutputBoundary() {
  const normal = fixture("normal.md");
  const ref = await ensureNormalRef(normal);
  const { json: readableRead } = await runSuccessfulJsonCase(
    "MD-OUTPUT-001 read normal readable-json",
    ["read", normal, "--ref", ref, "--output", "readable-json"],
    {
      schema: "readableRead",
      check: expectNoProtocolEnvelope
    }
  );

  const readableView = await runCli("MD-OUTPUT-001 read normal readable-view", [
    "read",
    normal,
    "--ref",
    ref,
    "--output",
    "readable-view"
  ]);
  expectExit(readableView, 0);
  expectStderrEmpty(readableView);
  expect(readableView, readableView.stdout.trimStart().startsWith("{"), "readable-view stdout starts with JSON header");
  expect(readableView, !readableView.stdout.includes("\"protocol_version\""), "readable-view stdout omits protocol envelope");
  const header = parseReadableViewHeader(readableView);
  expect(readableView, header.content?.$block === "/content", "read header has $block reference");
  expectReadableViewBlockRestoresField(readableView, readableView.stdout, "/content", readableRead.content);

  const defaultOutput = await runCli("MD-OUTPUT-001 read normal default output", ["read", normal, "--ref", ref]);
  expectExit(defaultOutput, 0);
  expectStderrEmpty(defaultOutput);
  expect(defaultOutput, defaultOutput.stdout.trimStart().startsWith("{"), "default output starts with JSON header (readable-view)");
  expect(defaultOutput, !defaultOutput.stdout.includes("\"protocol_version\""), "default output omits protocol envelope");

  await runProtocolResponseCase(
    "MD-OUTPUT-001 read normal protocol-json",
    ["read", normal, "--ref", ref, "--output", "protocol-json"],
    {
      operation: "read",
      check: (record: ExternalValue, json: ExternalValue) => {
        expect(record, json.result.ref === ref, "read protocol result preserves ref");
        expect(record, json.result.content_type === "text/markdown", "read protocol result has content_type");
        expectReadResultsEquivalent(record, json.result, readableRead, "read protocol-json result matches readable-json");
      }
    }
  );
}

async function ensureNormalRef(normal: ExternalValue) {
  const { record, json } = await runSuccessfulJsonCase(
    "MD-OUTPUT-001 outline normal readable-json for ref",
    ["outline", normal, "--output", "readable-json"],
    {
      schema: "readableOutline",
      check: expectNoProtocolEnvelope
    }
  );
  const ref = json.entries[0].ref;
  setNormalRef(ref);
  expect(record, typeof ref === "string" && ref.length > 0, "outline exposes a nonempty ref");
  return ref;
}
