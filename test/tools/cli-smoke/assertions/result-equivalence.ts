import type { CommandRecord } from "../../smoke-harness.ts";
import {
  expect,
  expectJsonObject,
  expectObjectArray
} from "./base.ts";

type EntryListExpectation = { field: "entries" | "matches"; itemLabel: "entry" | "match" };

const readResultFields = ["ref", "content", "content_type", "cost", "page"] as const;

export function expectOutlineResultsEquivalent(record: CommandRecord, actual: unknown, expected: unknown, summary: string) {
  expectEntryListEquivalent(record, actual, expected, summary, { field: "entries", itemLabel: "entry" });
}

export function expectReadResultsEquivalent(record: CommandRecord, actual: unknown, expected: unknown, summary: string) {
  const actualObject = readableReadResult(record, actual, `${summary}: actual`);
  const expectedObject = expectJsonObject(record, expected, `${summary}: expected is an object`);
  for (const field of readResultFields) {
    expect(record, actualObject[field] === expectedObject[field], `${summary}: ${field} matches`);
  }
}

export function expectFindResultsEquivalent(record: CommandRecord, actual: unknown, expected: unknown, summary: string) {
  expectEntryListEquivalent(record, actual, expected, summary, { field: "matches", itemLabel: "match" });
}

export function expectInfoResultsEquivalent(record: CommandRecord, actual: unknown, expected: unknown, summary: string) {
  const actualObject = readableInfoResult(record, actual, `${summary}: actual`);
  const expectedObject = expectJsonObject(record, expected, `${summary}: expected is an object`);
  expect(record, actualObject.display === expectedObject.display, `${summary}: display matches`);
  expect(
    record,
    JSON.stringify(actualObject.capabilities) === JSON.stringify(expectedObject.capabilities),
    `${summary}: capabilities match`
  );
}

function expectEntryListEquivalent(
  record: CommandRecord,
  actual: unknown,
  expected: unknown,
  summary: string,
  expectation: EntryListExpectation
) {
  const actualObject = readableEntryListResult(record, actual, `${summary}: actual`, expectation);
  const expectedObject = expectJsonObject(record, expected, `${summary}: expected is an object`);
  const actualItems = expectObjectArray(record, actualObject[expectation.field], `${summary}: actual ${expectation.field} are objects`);
  const expectedItems = expectObjectArray(record, expectedObject[expectation.field], `${summary}: expected ${expectation.field} are objects`);
  expect(record, actualObject.page === expectedObject.page, `${summary}: page matches`);
  expect(record, actualItems.length === expectedItems.length, `${summary}: ${expectation.itemLabel} count matches`);
  for (const index of actualItems.keys()) {
    const actualItem = actualItems[index];
    const expectedItem = expectedItems[index];
    expect(record, actualItem.ref === expectedItem.ref, `${summary}: ${expectation.itemLabel} ${index + 1} ref matches`);
    expect(record, actualItem.display === expectedItem.display, `${summary}: ${expectation.itemLabel} ${index + 1} display matches`);
  }
}

function readableReadResult(record: CommandRecord, value: unknown, label: string): Record<string, unknown> {
  const object = expectJsonObject(record, value, `${label} is an object`);
  if (typeof object.cost === "string") {
    return object;
  }
  return {
    ref: object.ref,
    content: object.content,
    content_type: object.content_type,
    cost: costSummary(record, object.cost, `${label} cost`),
    page: object.page
  };
}

function readableEntryListResult(
  record: CommandRecord,
  value: unknown,
  label: string,
  expectation: EntryListExpectation
): Record<string, unknown> {
  const object = expectJsonObject(record, value, `${label} is an object`);
  const items = expectObjectArray(record, object[expectation.field], `${label} ${expectation.field} are objects`);
  if (items.every((item) => typeof item.display === "string")) {
    return object;
  }
  return {
    [expectation.field]: items.map((item) => readableEntry(record, item, `${label} ${expectation.itemLabel}`)),
    page: object.page
  };
}

function readableEntry(record: CommandRecord, value: unknown, label: string): Record<string, unknown> {
  const entry = expectJsonObject(record, value, `${label} is an object`);
  return {
    ref: entry.ref,
    display: entryDisplay(record, entry, label)
  };
}

function entryDisplay(record: CommandRecord, entry: Record<string, unknown>, label: string): string {
  switch (entry.kind) {
    case "heading":
      return headingDisplay(record, entry, label);
    case "match":
      return matchDisplay(record, entry, label);
    case "document":
      return labeledCostDisplay(record, entry, label);
    default:
      return genericEntryDisplay(record, entry, label);
  }
}

function headingDisplay(record: CommandRecord, entry: Record<string, unknown>, label: string): string {
  const entryLabel = expectStringValue(record, entry.label, `${label} label`);
  const metadata = entry.metadata === undefined ? undefined : expectJsonObject(record, entry.metadata, `${label} metadata`);
  const level = metadata?.heading_level;
  const display = typeof level === "number" ? `H${level} ${entryLabel}` : entryLabel;
  return entry.cost === undefined ? display : `${display} | ${costSummary(record, entry.cost, `${label} cost`)}`;
}

function matchDisplay(record: CommandRecord, entry: Record<string, unknown>, label: string): string {
  const entryLabel = expectStringValue(record, entry.label, `${label} label`);
  if (entry.location === undefined) {
    return entryLabel;
  }
  const location = expectJsonObject(record, entry.location, `${label} location`);
  return typeof location.line_start === "number" ? `L${location.line_start}: ${entryLabel}` : entryLabel;
}

function labeledCostDisplay(record: CommandRecord, entry: Record<string, unknown>, label: string): string {
  const entryLabel = expectStringValue(record, entry.label, `${label} label`);
  return entry.cost === undefined ? entryLabel : `${entryLabel} | ${costSummary(record, entry.cost, `${label} cost`)}`;
}

function genericEntryDisplay(record: CommandRecord, entry: Record<string, unknown>, label: string): string {
  const entryLabel = expectStringValue(record, entry.label, `${label} label`);
  if (typeof entry.summary === "string") {
    return `${entryLabel} | ${entry.summary}`;
  }
  if (typeof entry.excerpt === "string") {
    return `${entryLabel} | ${entry.excerpt}`;
  }
  return entry.cost === undefined ? entryLabel : `${entryLabel} | ${costSummary(record, entry.cost, `${label} cost`)}`;
}

function readableInfoResult(record: CommandRecord, value: unknown, label: string): Record<string, unknown> {
  const object = expectJsonObject(record, value, `${label} is an object`);
  if (typeof object.display === "string") {
    return object;
  }
  return {
    display: infoDisplay(record, object, label),
    capabilities: object.capabilities
  };
}

function infoDisplay(record: CommandRecord, object: Record<string, unknown>, label: string): string {
  const parts: string[] = [];
  const adapter = object.adapter === undefined ? undefined : expectJsonObject(record, object.adapter, `${label} adapter`);
  const document = object.document === undefined ? undefined : expectJsonObject(record, object.document, `${label} document`);
  const metadata = object.metadata === undefined ? undefined : expectJsonObject(record, object.metadata, `${label} metadata`);
  if (typeof adapter?.format === "string") {
    parts.push(adapter.format === "markdown" ? "Markdown" : adapter.format);
  }
  if (typeof document?.content_type === "string") {
    parts.push(document.content_type);
  }
  if (typeof metadata?.heading_count === "number") {
    parts.push(`${metadata.heading_count} ${metadata.heading_count === 1 ? "heading" : "headings"}`);
  }
  if (document?.size !== undefined) {
    parts.push(measurementSummary(record, document.size, `${label} document size`));
  }
  return parts.length === 0 ? "document info" : parts.join(" | ");
}

function costSummary(record: CommandRecord, value: unknown, label: string): string {
  const cost = expectJsonObject(record, value, `${label} is an object`);
  const measurements = expectObjectArray(record, cost.measurements, `${label} measurements are objects`);
  return measurements
    .map((measurement) => costMeasurementSummary(record, measurement, `${label} measurement`))
    .join(" | ");
}

function costMeasurementSummary(record: CommandRecord, value: unknown, label: string): string {
  const measurement = expectJsonObject(record, value, `${label} is an object`);
  return measurement.unit === "byte" || measurement.unit === "bytes"
    ? `${(expectNumberValue(record, measurement.value, `${label} value`) / 1024).toFixed(1)} KB`
    : measurementSummary(record, measurement, label);
}

function measurementSummary(record: CommandRecord, value: unknown, label: string): string {
  const measurement = expectJsonObject(record, value, `${label} is an object`);
  const measurementValue = expectNumberValue(record, measurement.value, `${label} value`);
  switch (measurement.unit) {
    case "line":
    case "lines":
      return `${measurementValue} ${measurementValue === 1 ? "line" : "lines"}`;
    case "byte":
    case "bytes":
      return measurementValue < 1024 ? `${measurementValue} B` : `${(measurementValue / 1024).toFixed(1)} KB`;
    default:
      return `${measurementValue} ${expectStringValue(record, measurement.unit, `${label} unit`)}`;
  }
}

function expectStringValue(record: CommandRecord, value: unknown, label: string): string {
  expect(record, typeof value === "string", `${label} is a string`);
  return typeof value === "string" ? value : "";
}

function expectNumberValue(record: CommandRecord, value: unknown, label: string): number {
  expect(record, typeof value === "number", `${label} is a number`);
  return typeof value === "number" ? value : 0;
}
