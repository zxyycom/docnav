import { expect } from "../../tools/cli-smoke/assertions.ts";
import type { CommandRecord } from "../../tools/smoke-harness.ts";
import {
  expectedNormalFindDisplayKeywords,
  expectedNormalFindMatchCount
} from "./config.ts";

export * from "../../tools/cli-smoke/assertions.ts";

export function expectNormalFindResult(record: CommandRecord, result: unknown, label: string) {
  const resultRecord = isRecord(result) ? result : {};
  const matches = Array.isArray(resultRecord.matches) ? resultRecord.matches : [];
  expect(record, Array.isArray(resultRecord.matches), `${label} has matches array`);
  expect(
    record,
    matches.length === expectedNormalFindMatchCount,
    `${label} returns exactly ${expectedNormalFindMatchCount} matches`
  );
  const refs = matches.map((match) => (isRecord(match) ? match.ref : undefined));
  expect(
    record,
    refs.every((ref) => typeof ref === "string" && ref.length > 0),
    `${label} match refs are nonempty`
  );
  expect(record, new Set(refs).size === refs.length, `${label} match refs are unique`);
  for (const [index, match] of matches.entries()) {
    const actual = isRecord(match) ? match : {};
    expect(
      record,
      typeof actual.display === "string" && actual.display.length > 0,
      `${label} match ${index + 1} display is nonempty`
    );
    for (const keyword of expectedNormalFindDisplayKeywords) {
      expect(
        record,
        typeof actual.display === "string" && actual.display.includes(keyword),
        `${label} match ${index + 1} display includes ${keyword}`
      );
    }
  }
  expect(record, resultRecord.page === null, `${label} page is null`);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
