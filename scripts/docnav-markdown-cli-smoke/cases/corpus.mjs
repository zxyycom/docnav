import { fixture } from "../fixtures.mjs";
import { runCli, validateSchema } from "../harness.mjs";
import {
  expect,
  expectExit,
  expectStderrEmpty,
  parseJson
} from "../assertions.mjs";

export function testProcessBoundaryCorpus() {
  testUnicodePaging();
  testLargePagination();
  testBomAndCrlf();
}

function testUnicodePaging() {
  const unicode = fixture("unicode.md");
  const outline = runCli("outline unicode readable-json", [
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

  const full = runCli("read unicode full readable-json", [
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

  const paged = readAllPages(unicode, ref, 12, "unicode");
  expect(
    paged.lastRecord,
    paged.content === fullJson.content,
    "unicode paged reads reassemble the full content"
  );
}

function testLargePagination() {
  const large = fixture("large-pagination.md");
  const first = runCli("read large page 1 readable-json", [
    "read",
    large,
    "--ref",
    "doc:full",
    "--limit-chars",
    "120",
    "--page",
    "1",
    "--output",
    "readable-json"
  ]);
  expectExit(first, 0);
  expectStderrEmpty(first);
  const firstJson = parseJson(first);
  validateSchema(first, "readableRead", firstJson);
  expect(first, firstJson.page === 2, "large read page 1 returns continuation page 2");

  const second = runCli("read large continuation readable-json", [
    "read",
    large,
    "--ref",
    "doc:full",
    "--limit-chars",
    "120",
    "--page",
    String(firstJson.page),
    "--output",
    "readable-json"
  ]);
  expectExit(second, 0);
  expectStderrEmpty(second);
  const secondJson = parseJson(second);
  validateSchema(second, "readableRead", secondJson);
  expect(second, secondJson.content.length > 0, "large continuation returns content");
  expect(second, secondJson.content !== firstJson.content, "large continuation advances content");
  expect(
    second,
    secondJson.page === null || secondJson.page === firstJson.page + 1,
    "large continuation page is next page or null"
  );

  const beyond = runCli("read large beyond end readable-json", [
    "read",
    large,
    "--ref",
    "doc:full",
    "--limit-chars",
    "120",
    "--page",
    "999",
    "--output",
    "readable-json"
  ]);
  expectExit(beyond, 0);
  expectStderrEmpty(beyond);
  const beyondJson = parseJson(beyond);
  validateSchema(beyond, "readableRead", beyondJson);
  expect(beyond, beyondJson.content === "", "large beyond-end read returns empty content");
  expect(beyond, beyondJson.page === null, "large beyond-end read returns page null");

  const firstFind = runCli("find large page 1 readable-json", [
    "find",
    large,
    "--query",
    "target",
    "--limit-chars",
    "120",
    "--page",
    "1",
    "--output",
    "readable-json"
  ]);
  expectExit(firstFind, 0);
  expectStderrEmpty(firstFind);
  const firstFindJson = parseJson(firstFind);
  validateSchema(firstFind, "readableFind", firstFindJson);
  expect(firstFind, firstFindJson.matches.length > 0, "large find page 1 returns matches");
  expect(firstFind, firstFindJson.page === 2, "large find page 1 returns continuation page 2");

  const secondFind = runCli("find large continuation readable-json", [
    "find",
    large,
    "--query",
    "target",
    "--limit-chars",
    "120",
    "--page",
    String(firstFindJson.page),
    "--output",
    "readable-json"
  ]);
  expectExit(secondFind, 0);
  expectStderrEmpty(secondFind);
  const secondFindJson = parseJson(secondFind);
  validateSchema(secondFind, "readableFind", secondFindJson);
  expect(secondFind, secondFindJson.matches.length > 0, "large find continuation returns matches");
  expect(
    secondFind,
    JSON.stringify(secondFindJson.matches) !== JSON.stringify(firstFindJson.matches),
    "large find continuation advances matches"
  );
  expect(
    secondFind,
    secondFindJson.page === null || secondFindJson.page === firstFindJson.page + 1,
    "large find continuation page is next page or null"
  );

  const findBeyond = runCli("find large beyond end readable-json", [
    "find",
    large,
    "--query",
    "target",
    "--limit-chars",
    "120",
    "--page",
    "999",
    "--output",
    "readable-json"
  ]);
  expectExit(findBeyond, 0);
  expectStderrEmpty(findBeyond);
  const findBeyondJson = parseJson(findBeyond);
  validateSchema(findBeyond, "readableFind", findBeyondJson);
  expect(findBeyond, findBeyondJson.matches.length === 0, "large beyond-end find returns no matches");
  expect(findBeyond, findBeyondJson.page === null, "large beyond-end find returns page null");
}

function testBomAndCrlf() {
  const bom = runCli("outline UTF-8 BOM readable-json", [
    "outline",
    fixture("utf8-bom.md"),
    "--output",
    "readable-json"
  ]);
  expectExit(bom, 0);
  expectStderrEmpty(bom);
  const bomOutline = parseJson(bom);
  validateSchema(bom, "readableOutline", bomOutline);
  expect(bom, bomOutline.entries[0].ref === "L1:Bom Heading", "BOM fixture heading ref starts at line 1");

  const bomRead = runCli("read UTF-8 BOM readable-json", [
    "read",
    fixture("utf8-bom.md"),
    "--ref",
    bomOutline.entries[0].ref,
    "--output",
    "readable-json"
  ]);
  expectExit(bomRead, 0);
  expectStderrEmpty(bomRead);
  const bomReadJson = parseJson(bomRead);
  validateSchema(bomRead, "readableRead", bomReadJson);
  expect(bomRead, bomReadJson.content.startsWith("# Bom Heading"), "BOM is stripped before read content");

  const crlf = runCli("outline CRLF readable-json", [
    "outline",
    fixture("crlf.md"),
    "--output",
    "readable-json"
  ]);
  expectExit(crlf, 0);
  expectStderrEmpty(crlf);
  const crlfOutline = parseJson(crlf);
  validateSchema(crlf, "readableOutline", crlfOutline);
  expect(crlf, crlfOutline.entries[0].ref === "L1:Crlf Heading", "CRLF fixture heading ref starts at line 1");

  const crlfRead = runCli("read CRLF readable-json", [
    "read",
    fixture("crlf.md"),
    "--ref",
    crlfOutline.entries[0].ref,
    "--output",
    "readable-json"
  ]);
  expectExit(crlfRead, 0);
  expectStderrEmpty(crlfRead);
  const crlfReadJson = parseJson(crlfRead);
  validateSchema(crlfRead, "readableRead", crlfReadJson);
  expect(crlfRead, crlfReadJson.content.includes("\r\n"), "CRLF read preserves CRLF content");
}

function readAllPages(documentPath, ref, limitChars, label) {
  let page = 1;
  let content = "";
  let lastRecord = null;
  for (let count = 0; count < 20; count += 1) {
    const record = runCli(`read ${label} page ${page} readable-json`, [
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
