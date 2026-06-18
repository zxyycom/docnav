import { fixture } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectStderrEmpty,
  parseJson
} from "../assertions.ts";

export function createProcessBoundaryCorpusTasks() {
  return [
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
  const ref = outlineJson.entries[0].ref;

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
  expect(full, fullJson.content.includes("世界"), "unicode full read includes CJK text");
  expect(full, fullJson.content.includes("🙂"), "unicode full read includes emoji");
  expect(full, !fullJson.content.includes("\uFFFD"), "unicode full read has no replacement characters");

  const paged = await readAllPages(unicode, ref, 12, "MD-CORPUS-001 unicode");
  expect(paged.lastRecord, paged.content === fullJson.content, "unicode paged reads reassemble the full content");
}

async function readAllPages(documentPath: ExternalValue, ref: ExternalValue, limitChars: ExternalValue, label: ExternalValue) {
  let page = 1;
  let content = "";
  let lastRecord;
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
    lastRecord = record;
    expectExit(record, 0);
    expectStderrEmpty(record);
    const json = parseJson(record);
    validateSchema(record, "readableRead", json);
    content += json.content;
    if (json.page === null) {
      return { content, lastRecord };
    }
    expect(record, json.page === page + 1, `${label} page advances by one`);
    page = json.page;
  }
  throw new Error(`${label} pagination did not terminate`);
}
