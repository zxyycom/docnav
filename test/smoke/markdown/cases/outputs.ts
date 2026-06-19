import { fixture, setNormalRef } from "../fixtures.ts";
import { runCli, runProtocolResponseCase, runSuccessfulJsonCase } from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectNoProtocolEnvelope,
  expectObjectArray,
  expectReadableViewBlockRestoresField,
  expectReadResultsEquivalent,
  expectStderrEmpty,
  expectString,
  expectStringArray,
  parseReadableViewHeader
} from "../assertions.ts";

export function createDocumentLinkTasks() {
  return [
    // @case BB-MD-LINK-001
    {
      id: "MD-LINK-001",
      label: "MD-LINK-001 markdown outline find read info chain",
      run: testMarkdownDocumentLinkChain
    }
  ];
}

export function createDocumentOutputBoundaryTasks() {
  return [
    // @case BB-MD-OUTPUT-001
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
      check: (record, json) => {
        expectNoProtocolEnvelope(record, json);
        const entries = expectObjectArray(record, json.entries, "outline entries are objects");
        const refs = entries.map((entry) => entry.ref);
        const firstDisplay = expectString(record, entries[0]?.display, "outline first entry display is a string");
        expect(record, entries.length > 0, "outline returns entries");
        expect(record, json.page === null, "outline page is null for normal fixture");
        expect(record, new Set(refs).size === entries.length, "outline refs are unique");
        expect(record, firstDisplay.includes("H1"), "outline first entry identifies a top-level heading");
      }
    }
  );
  const outlineEntries = expectObjectArray(outlineRecord, outline.entries, "outline entries are objects");
  const outlineRef = expectString(outlineRecord, outlineEntries[0]?.ref, "outline first ref is a string");
  setNormalRef(outlineRef);
  expect(outlineRecord, outlineRef.length > 0, "outline exposes a nonempty ref");

  const { json: read } = await runSuccessfulJsonCase(
    "MD-LINK-001 read outline ref readable-json",
    ["read", normal, "--ref", outlineRef, "--output", "readable-json"],
    {
      schema: "readableRead",
      check: (record, json) => {
        const content = expectString(record, json.content, "read content is a string");
        expectNoProtocolEnvelope(record, json);
        expect(record, json.ref === outlineRef, "read result preserves outline ref");
        expect(record, content.includes("# Guide"), "read content includes heading");
        expect(record, content.includes("target text"), "read content includes target text");
        expect(record, json.content_type === "text/markdown", "read content_type is text/markdown");
        expect(record, json.page === null, "read page is null for normal fixture");
      }
    }
  );

  const { record: findRecord, json: find } = await runSuccessfulJsonCase(
    "MD-LINK-001 find target readable-json",
    ["find", normal, "--query", "target", "--output", "readable-json"],
    {
      schema: "readableFind",
      check: (record, json) => {
        const matches = expectObjectArray(record, json.matches, "find matches are objects");
        const firstRef = expectString(record, matches[0]?.ref, "find first match ref is a string");
        const firstDisplay = expectString(record, matches[0]?.display, "find first match display is a string");
        expectNoProtocolEnvelope(record, json);
        expect(record, matches.length > 0, "find returns matches");
        expect(record, firstRef.length > 0, "find exposes ref");
        expect(record, firstDisplay.includes("target"), "find display includes query text");
      }
    }
  );
  const findMatches = expectObjectArray(findRecord, find.matches, "find matches are objects");
  const findRef = expectString(findRecord, findMatches[0]?.ref, "find first match ref is a string");

  await runSuccessfulJsonCase(
    "MD-LINK-001 read find ref readable-json",
    ["read", normal, "--ref", findRef, "--output", "readable-json"],
    {
      schema: "readableRead",
      check: (record, json) => {
        expectNoProtocolEnvelope(record, json);
        expect(record, json.ref === findRef, "read preserves find ref");
        expect(record, json.content_type === read.content_type, "read from find ref preserves content_type");
      }
    }
  );

  await runSuccessfulJsonCase("MD-LINK-001 info normal readable-json", ["info", normal, "--output", "readable-json"], {
    schema: "readableInfo",
    check: (record, json) => {
      const display = expectString(record, json.display, "info display is a string");
      const capabilities = expectStringArray(record, json.capabilities, "info capabilities are strings");
      expectNoProtocolEnvelope(record, json);
      expect(record, display.includes("Markdown | text/markdown"), "info readable result has Markdown display");
      for (const capability of ["outline", "read", "find", "info"]) {
        expect(record, capabilities.includes(capability), `info readable includes ${capability} capability`);
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
  const blockRef = expectJsonObject(readableView, header.content, "read header content block reference is an object");
  const readableContent = expectString(readableView, readableRead.content, "readable-json content is a string");
  expect(readableView, blockRef.$block === "/content", "read header has $block reference");
  expectReadableViewBlockRestoresField(readableView, readableView.stdout, "/content", readableContent);

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
      check: (record, json) => {
        const result = expectJsonObject(record, json.result, "read protocol result is an object");
        expect(record, result.ref === ref, "read protocol result preserves ref");
        expect(record, result.content_type === "text/markdown", "read protocol result has content_type");
        expectReadResultsEquivalent(record, result, readableRead, "read protocol-json result matches readable-json");
      }
    }
  );
}

async function ensureNormalRef(normal: string) {
  const { record, json } = await runSuccessfulJsonCase(
    "MD-OUTPUT-001 outline normal readable-json for ref",
    ["outline", normal, "--output", "readable-json"],
    {
      schema: "readableOutline",
      check: expectNoProtocolEnvelope
    }
  );
  const entries = expectObjectArray(record, json.entries, "outline entries are objects");
  const ref = expectString(record, entries[0]?.ref, "outline first ref is a string");
  setNormalRef(ref);
  expect(record, ref.length > 0, "outline exposes a nonempty ref");
  return ref;
}
