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
  const actualObject = expectJsonObject(record, actual, `${summary}: actual is an object`);
  const expectedObject = expectJsonObject(record, expected, `${summary}: expected is an object`);
  for (const field of readResultFields) {
    expect(record, actualObject[field] === expectedObject[field], `${summary}: ${field} matches`);
  }
}

export function expectFindResultsEquivalent(record: CommandRecord, actual: unknown, expected: unknown, summary: string) {
  expectEntryListEquivalent(record, actual, expected, summary, { field: "matches", itemLabel: "match" });
}

export function expectInfoResultsEquivalent(record: CommandRecord, actual: unknown, expected: unknown, summary: string) {
  const actualObject = expectJsonObject(record, actual, `${summary}: actual is an object`);
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
  const actualObject = expectJsonObject(record, actual, `${summary}: actual is an object`);
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
