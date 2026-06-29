import { assert, assertDeepEqual } from "../../assertions.ts";
import {
  FIELDS,
  OPERATIONS,
  OPERATION_NAMES,
  READABLE_EXAMPLE_FILE,
} from "../../config.ts";
import { readJson } from "../../json/files.ts";
import { validateExampleBudget } from "./budget.ts";
import { jsonArray, jsonObject } from "./json.ts";
import { validateProtocolPair } from "./protocol-pairs.ts";

function toReadablePayload(operation: string, protocolResult: unknown): unknown {
  const result = jsonObject(protocolResult, `${operation} protocol result`);
  switch (operation) {
    case OPERATION_NAMES.outline:
      return {
        entries: jsonArray(result[FIELDS.entries], "outline entries").map(readableEntry),
        page: result[FIELDS.page],
      };
    case OPERATION_NAMES.read:
      return {
        ref: result[FIELDS.ref],
        content: result[FIELDS.content],
        content_type: result[FIELDS.contentType],
        cost: costSummary(jsonObject(result[FIELDS.cost], "read cost")),
        page: result[FIELDS.page],
      };
    case OPERATION_NAMES.find:
      return {
        matches: jsonArray(result[FIELDS.matches], "find matches").map(readableEntry),
        page: result[FIELDS.page],
      };
    case OPERATION_NAMES.info:
      return {
        display: infoDisplay(result),
        capabilities: result[FIELDS.capabilities],
      };
    default:
      throw new Error(`unknown operation ${operation}`);
  }
}

function readableEntry(value: unknown): Record<string, unknown> {
  const entry = jsonObject(value, "protocol entry");
  return {
    ref: entry[FIELDS.ref],
    display: entryDisplay(entry),
  };
}

function entryDisplay(entry: Record<string, unknown>): string {
  switch (entry.kind) {
    case "heading":
      return headingDisplay(entry);
    case "match":
      return matchDisplay(entry);
    case "document":
      return labeledCostDisplay(entry);
    default:
      return genericEntryDisplay(entry);
  }
}

function headingDisplay(entry: Record<string, unknown>): string {
  const label = stringField(entry.label, "entry label");
  const metadata = entry.metadata === undefined ? undefined : jsonObject(entry.metadata, "entry metadata");
  const level = metadata?.heading_level;
  const prefix = typeof level === "number" ? `H${level} ${label}` : label;
  const cost = entry.cost === undefined ? undefined : costSummary(jsonObject(entry.cost, "entry cost"));
  return cost === undefined ? prefix : `${prefix} | ${cost}`;
}

function matchDisplay(entry: Record<string, unknown>): string {
  const label = stringField(entry.label, "entry label");
  if (entry.location === undefined) {
    return label;
  }
  const location = jsonObject(entry.location, "entry location");
  return typeof location.line_start === "number"
    ? `L${location.line_start}: ${label}`
    : label;
}

function labeledCostDisplay(entry: Record<string, unknown>): string {
  const label = stringField(entry.label, "entry label");
  const cost = entry.cost === undefined ? undefined : costSummary(jsonObject(entry.cost, "entry cost"));
  return cost === undefined ? label : `${label} | ${cost}`;
}

function genericEntryDisplay(entry: Record<string, unknown>): string {
  const label = stringField(entry.label, "entry label");
  if (typeof entry.summary === "string") {
    return `${label} | ${entry.summary}`;
  }
  if (typeof entry.excerpt === "string") {
    return `${label} | ${entry.excerpt}`;
  }
  if (entry.cost !== undefined) {
    return `${label} | ${costSummary(jsonObject(entry.cost, "entry cost"))}`;
  }
  return label;
}

function infoDisplay(result: Record<string, unknown>): string {
  const adapter = result.adapter === undefined ? undefined : jsonObject(result.adapter, "info adapter");
  const document = result.document === undefined ? undefined : jsonObject(result.document, "info document");
  const metadata = result.metadata === undefined ? undefined : jsonObject(result.metadata, "info metadata");
  const parts = compactParts([
    adapterFormatDisplay(adapter),
    documentContentTypeDisplay(document),
    headingCountDisplay(metadata),
    documentSizeDisplay(document),
  ]);
  return parts.length === 0 ? "document info" : parts.join(" | ");
}

function adapterFormatDisplay(adapter: Record<string, unknown> | undefined): string | undefined {
  if (typeof adapter?.format === "string") {
    return adapter.format === "markdown" ? "Markdown" : adapter.format;
  }
  return undefined;
}

function documentContentTypeDisplay(document: Record<string, unknown> | undefined): string | undefined {
  if (typeof document?.content_type === "string") {
    return document.content_type;
  }
  return undefined;
}

function headingCountDisplay(metadata: Record<string, unknown> | undefined): string | undefined {
  if (typeof metadata?.heading_count === "number") {
    const label = metadata.heading_count === 1 ? "heading" : "headings";
    return `${metadata.heading_count} ${label}`;
  }
  return undefined;
}

function documentSizeDisplay(document: Record<string, unknown> | undefined): string | undefined {
  if (document?.size !== undefined) {
    return measurementSummary(jsonObject(document.size, "document size"));
  }
  return undefined;
}

function compactParts(parts: Array<string | undefined>): string[] {
  return parts.filter((part): part is string => part !== undefined);
}

function costSummary(cost: Record<string, unknown>): string {
  const measurements = jsonArray(cost.measurements, "cost measurements");
  assert(measurements.length > 0, "cost measurements must be non-empty");
  return measurements
    .map((measurement) => costMeasurementSummary(jsonObject(measurement, "cost measurement")))
    .join(" | ");
}

function costMeasurementSummary(measurement: Record<string, unknown>): string {
  return measurement.unit === "byte" || measurement.unit === "bytes"
    ? `${(numberField(measurement.value, "measurement value") / 1024).toFixed(1)} KB`
    : measurementSummary(measurement);
}

function measurementSummary(measurement: Record<string, unknown>): string {
  const value = numberField(measurement.value, "measurement value");
  switch (measurement.unit) {
    case "line":
    case "lines":
      return `${value} ${value === 1 ? "line" : "lines"}`;
    case "byte":
    case "bytes":
      return value < 1024 ? `${value} B` : `${(value / 1024).toFixed(1)} KB`;
    default:
      return `${value} ${stringField(measurement.unit, "measurement unit")}`;
  }
}

function stringField(value: unknown, label: string): string {
  assert(typeof value === "string", `${label} must be a string`);
  return value;
}

function numberField(value: unknown, label: string): number {
  assert(typeof value === "number", `${label} must be a number`);
  return value;
}

export function validateProtocolReadableMappings() {
  for (const operation of OPERATIONS) {
    const { request, response } = validateProtocolPair(operation);
    const result = jsonObject(response[FIELDS.result], `${operation} response result`);
    validateExampleBudget(operation, request, result);

    const readable = readJson(READABLE_EXAMPLE_FILE.result(operation));
    assertDeepEqual(
      readable,
      toReadablePayload(operation, response[FIELDS.result]),
      `${operation} readable JSON must preserve protocol result semantics`,
    );
  }

  console.log(
    `protocol/readable mapping ok: ${OPERATIONS.length} operation(s)`,
  );
}
