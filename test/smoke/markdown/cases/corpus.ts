import { fixture } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import type { CommandRecord } from "../../../tools/smoke-harness.ts";
import {
  expect,
  expectExit,
  expectObjectArray,
  expectStderrEmpty,
  expectString,
  expectNumber,
  parseJson
} from "../assertions.ts";

export function createProcessBoundaryCorpusTasks() {
  return [
    // @case BB-MD-CORPUS-001
    {
      id: "MD-CORPUS-001",
      label: "MD-CORPUS-001 unicode pagination process boundary",
      run: testUnicodePagination
    }
  ];
}

async function testUnicodePagination() {
  const unicode = fixture("unicode.md");
  const outline = await runCli("MD-CORPUS-001 outline unicode readable-json", [
    "outline",
    unicode,
    "--output",
    "readable-json"
  ]);
  expectExit(outline, 0);
  expectStderrEmpty(outline);
  const outlineJson = parseJson(outline);
  validateSchema(outline, "readableOutline", outlineJson);
  const entries = expectObjectArray(outline, outlineJson.entries, "outline entries are objects");
  const ref = expectString(outline, entries[0]?.ref, "outline first entry ref is a string");

  const full = await runCli("MD-CORPUS-001 read unicode full readable-json", [
    "read",
    unicode,
    "--ref",
    ref,
    "--output",
    "readable-json"
  ]);
  expectExit(full, 0);
  expectStderrEmpty(full);
  const fullJson = parseJson(full);
  validateSchema(full, "readableRead", fullJson);
  const fullContent = expectString(full, fullJson.content, "full read content is a string");
  expect(full, fullContent.includes("世界"), "unicode full read includes CJK text");
  expect(full, fullContent.includes("🙂"), "unicode full read includes emoji");
  expect(full, !fullContent.includes("\uFFFD"), "unicode full read has no replacement characters");

  const paged = await readAllPages(unicode, ref, 12, "MD-CORPUS-001 unicode");
  expect(paged.lastRecord, paged.content === fullContent, "unicode paged reads reassemble the full content");
}

async function readAllPages(
  documentPath: string,
  ref: string,
  limitChars: number,
  label: string
): Promise<{ content: string; lastRecord: CommandRecord }> {
  let page = 1;
  let content = "";
  for (let count = 0; count < 20; count += 1) {
    const record = await runCli(`${label} page ${page} readable-json`, [
      "read",
      documentPath,
      "--ref",
      ref,
      "--limit-chars",
      String(limitChars),
      "--page",
      String(page),
      "--output",
      "readable-json"
    ]);
    expectExit(record, 0);
    expectStderrEmpty(record);
    const json = parseJson(record);
    validateSchema(record, "readableRead", json);
    const chunk = expectString(record, json.content, `${label} page content is a string`);
    content += chunk;
    if (json.page === null) {
      return { content, lastRecord: record };
    }
    expect(record, json.page === page + 1, `${label} page advances by one`);
    page = expectNumber(record, json.page, `${label} next page is a number`);
  }
  throw new Error(`${label} pagination did not terminate`);
}
