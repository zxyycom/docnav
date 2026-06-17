import { expect } from "../../tools/cli-smoke/assertions.mjs";
import {
  expectedNormalFindDisplayKeywords,
  expectedNormalFindMatchCount
} from "./config.mjs";

export * from "../../tools/cli-smoke/assertions.mjs";

export function expectNormalFindResult(record, result, label) {
  expect(record, Array.isArray(result.matches), `${label} has matches array`);
  expect(
    record,
    result.matches.length === expectedNormalFindMatchCount,
    `${label} returns exactly ${expectedNormalFindMatchCount} matches`
  );
  const refs = result.matches.map((match) => match?.ref);
  expect(
    record,
    refs.every((ref) => typeof ref === "string" && ref.length > 0),
    `${label} match refs are nonempty`
  );
  expect(record, new Set(refs).size === refs.length, `${label} match refs are unique`);
  for (const [index, actual] of result.matches.entries()) {
    expect(
      record,
      typeof actual.display === "string" && actual.display.length > 0,
      `${label} match ${index + 1} display is nonempty`
    );
    for (const keyword of expectedNormalFindDisplayKeywords) {
      expect(
        record,
        actual.display.includes(keyword),
        `${label} match ${index + 1} display includes ${keyword}`
      );
    }
  }
  expect(record, result.page === null, `${label} page is null`);
}
