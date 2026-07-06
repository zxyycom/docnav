import type { CommandRecord } from "../../../tools/smoke-harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectObjectArray,
  expectProtocolSuccess,
  expectReadableViewBlockRestoresField,
  expectReadableViewFieldValue,
  expectStderrEmpty,
  expectString,
  parseJson,
  parseReadableViewHeader
} from "../assertions.ts";
import { copyDocumentFixture } from "../fixtures.ts";
import type { SmokeProject } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import { runReadableJsonCase } from "./readable-output.ts";

const DOCUMENT_HEAD_FIXTURE = "document-head.md";
const DOCUMENT_HEAD_REF = "HEAD:leading";
const DOCUMENT_HEAD_CONTENT = [
  "---",
  "title: Document Head Fixture",
  "summary: Keep YAML delimiters.",
  "---",
  "",
  "Preface target text before the first heading.",
  "It should roundtrip through HEAD:leading.",
  "",
  ""
].join("\n");

// @case BB-CORE-MD-DOCHEAD-001
export async function assertDocumentHeadOutputModes(project: SmokeProject) {
  const documentPath = copyDocumentFixture(project, DOCUMENT_HEAD_FIXTURE, "docs/document-head.md");

  const readableOutline = await runReadableJsonCase(
    project,
    "CORE-LINK-001 document head outline readable-json",
    [
      "outline",
      documentPath,
      "--output",
      "readable-json"
    ],
    "readableOutline"
  );
  const readableHeadEntry = assertReadableDocumentHeadEntry(readableOutline.record, readableOutline.json);

  const protocolOutline = await runCli("CORE-LINK-001 document head outline protocol-json", [
    "outline",
    documentPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(protocolOutline, 0);
  expectStderrEmpty(protocolOutline);
  const protocolOutlineJson = parseJson(protocolOutline);
  validateSchema(protocolOutline, "protocolResponse", protocolOutlineJson);
  expectProtocolSuccess(protocolOutline, protocolOutlineJson, "outline");
  const protocolOutlineResult = expectJsonObject(
    protocolOutline,
    protocolOutlineJson.result,
    "document head protocol outline result is an object"
  );
  const protocolHeadEntry = assertProtocolDocumentHeadEntry(protocolOutline, protocolOutlineResult);
  assertDocumentHeadDisplayUsesProtocolFacts(
    readableOutline.record,
    readableHeadEntry,
    protocolHeadEntry,
    "document head readable-json derives display from protocol facts"
  );

  const readableViewOutline = await runCli("CORE-LINK-001 document head outline readable-view", [
    "outline",
    documentPath,
    "--output",
    "readable-view"
  ], { project });
  expectExit(readableViewOutline, 0);
  expectStderrEmpty(readableViewOutline);
  const readableViewHeader = parseReadableViewHeader(readableViewOutline);
  const readableViewHeadEntry = assertReadableDocumentHeadEntry(readableViewOutline, readableViewHeader);
  assertDocumentHeadDisplayUsesProtocolFacts(
    readableViewOutline,
    readableViewHeadEntry,
    protocolHeadEntry,
    "document head readable-view derives display from protocol facts"
  );

  const readableRead = await runReadableJsonCase(
    project,
    "CORE-LINK-001 document head read readable-json",
    [
      "read",
      documentPath,
      "--ref",
      DOCUMENT_HEAD_REF,
      "--output",
      "readable-json"
    ],
    "readableRead"
  );
  assertDocumentHeadReadPayload(readableRead.record, readableRead.json);

  const protocolRead = await runCli("CORE-LINK-001 document head read protocol-json", [
    "read",
    documentPath,
    "--ref",
    DOCUMENT_HEAD_REF,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(protocolRead, 0);
  expectStderrEmpty(protocolRead);
  const protocolReadJson = parseJson(protocolRead);
  validateSchema(protocolRead, "protocolResponse", protocolReadJson);
  expectProtocolSuccess(protocolRead, protocolReadJson, "read");
  const protocolReadResult = expectJsonObject(
    protocolRead,
    protocolReadJson.result,
    "document head protocol read result is an object"
  );
  assertDocumentHeadReadPayload(protocolRead, protocolReadResult);

  const readableViewRead = await runCli("CORE-LINK-001 document head read readable-view", [
    "read",
    documentPath,
    "--ref",
    DOCUMENT_HEAD_REF,
    "--output",
    "readable-view"
  ], { project });
  expectExit(readableViewRead, 0);
  expectStderrEmpty(readableViewRead);
  expectReadableViewFieldValue(readableViewRead, readableViewRead.stdout, "/ref", DOCUMENT_HEAD_REF);
  expectReadableViewFieldValue(readableViewRead, readableViewRead.stdout, "/content_type", "text/markdown");
  expectReadableViewBlockRestoresField(readableViewRead, readableViewRead.stdout, "/content", DOCUMENT_HEAD_CONTENT);
}

function assertReadableDocumentHeadEntry(record: CommandRecord, result: Record<string, unknown>) {
  expect(record, result.kind === "structured", "document head outline remains structured");
  const entries = expectObjectArray(record, result.entries, "document head readable entries are objects");
  expect(record, entries.length > 1, "document head outline includes head and heading entries");
  const headEntry = expectJsonObject(record, entries[0], "document head readable entry is first");
  expect(record, headEntry.ref === DOCUMENT_HEAD_REF, "document head readable entry uses HEAD ref");
  const display = expectString(record, headEntry.display, "document head readable entry has display");
  expect(record, display.length > 0, "document head readable display is nonempty");
  return headEntry;
}

function assertProtocolDocumentHeadEntry(record: CommandRecord, result: Record<string, unknown>) {
  expect(record, result.kind === "structured", "document head protocol outline remains structured");
  expect(record, !Object.hasOwn(result, "frontmatter"), "protocol outline omits top-level frontmatter");
  expect(record, !Object.hasOwn(result, "metadata"), "protocol outline omits top-level metadata");
  expect(record, !Object.hasOwn(result, "document_head"), "protocol outline omits top-level document_head");
  const entries = expectObjectArray(record, result.entries, "document head protocol entries are objects");
  expect(record, entries.length > 1, "document head protocol outline includes head and heading entries");
  const headEntry = expectJsonObject(record, entries[0], "document head protocol entry is first");
  expect(record, headEntry.ref === DOCUMENT_HEAD_REF, "document head protocol entry uses HEAD ref");
  const label = expectString(record, headEntry.label, "document head protocol entry has label");
  expect(record, label.length > 0, "document head protocol label is nonempty");
  const kind = expectString(record, headEntry.kind, "document head protocol entry has kind");
  expect(record, kind.length > 0, "document head protocol kind is nonempty");
  expect(record, kind !== "heading", "document head protocol kind is not heading");
  const location = expectJsonObject(record, headEntry.location, "document head protocol entry has location");
  expect(record, location.line_start === 1, "document head protocol location starts on line 1");
  const metadata = expectJsonObject(record, headEntry.metadata, "document head protocol entry has metadata");
  expect(record, Object.keys(metadata).length > 0, "document head protocol metadata is nonempty");
  expect(record, metadata.document_region === "leading", "document head protocol metadata uses document_region");
  expect(record, !Object.hasOwn(metadata, "region"), "document head protocol metadata omits legacy region");
  expect(record, !Object.hasOwn(headEntry, "display"), "protocol document head entry omits readable display");
  return headEntry;
}

function assertDocumentHeadReadPayload(record: CommandRecord, result: Record<string, unknown>) {
  expect(record, result.ref === DOCUMENT_HEAD_REF, "document head read preserves HEAD ref");
  expect(record, result.content === DOCUMENT_HEAD_CONTENT, "document head read returns original leading Markdown");
  expect(record, result.content_type === "text/markdown", "document head read content_type is markdown");
}

function assertDocumentHeadDisplayUsesProtocolFacts(
  record: CommandRecord,
  readableEntry: Record<string, unknown>,
  protocolEntry: Record<string, unknown>,
  summary: string
) {
  const display = expectString(record, readableEntry.display, `${summary}: readable display is a string`);
  const label = expectString(record, protocolEntry.label, `${summary}: protocol label is a string`);
  expect(record, display.startsWith(`${label} | `), `${summary}: display starts with raw label`);

  const measurements = protocolCostMeasurements(record, protocolEntry, summary);
  const lineCount = measurementValue(record, measurements, ["line", "lines"], summary);
  const byteCount = measurementValue(record, measurements, ["byte", "bytes"], summary);
  const tokenCount = measurementValue(record, measurements, ["token", "tokens"], summary);
  expect(record, display.includes(`${lineCount} ${lineCount === 1 ? "line" : "lines"}`), `${summary}: display includes raw line cost`);
  expect(record, display.includes(`${(byteCount / 1024).toFixed(1)} KB`), `${summary}: display includes raw byte cost`);
  expect(record, display.includes(`${tokenCount} ${tokenCount === 1 ? "token" : "tokens"}`), `${summary}: display includes raw token cost`);
}

function protocolCostMeasurements(
  record: CommandRecord,
  protocolEntry: Record<string, unknown>,
  summary: string
) {
  const cost = expectJsonObject(record, protocolEntry.cost, `${summary}: protocol entry has cost`);
  return expectObjectArray(record, cost.measurements, `${summary}: protocol cost measurements are objects`);
}

function measurementValue(
  record: CommandRecord,
  measurements: Record<string, unknown>[],
  units: string[],
  summary: string
) {
  const measurement = measurements.find((item) => units.includes(String(item.unit)));
  expect(record, measurement !== undefined, `${summary}: protocol cost has ${units.join("/")} measurement`);
  const value = measurement?.value;
  expect(record, typeof value === "number", `${summary}: protocol cost ${units[0]} value is numeric`);
  return typeof value === "number" ? value : 0;
}
