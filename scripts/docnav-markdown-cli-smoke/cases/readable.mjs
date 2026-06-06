import { fixture, setNormalReadableFindResult, setNormalReadableReadResult, setNormalRef } from "../fixtures.mjs";
import { runCli } from "../runner.mjs";
import {
  expect,
  expectExit,
  expectIncludes,
  expectNoProtocolEnvelope,
  expectNormalFindResult,
  expectStderrEmpty,
  parseJson
} from "../assertions.mjs";
import { validateSchema } from "../schemas.mjs";

export function testReadableOutlineRead() {
  const normal = fixture("normal.md");
  const outline = runCli("outline normal readable-json", [
    "outline",
    normal,
    "--output",
    "readable-json"
  ]);
  expectExit(outline, 0);
  expectStderrEmpty(outline);
  const outlineJson = parseJson(outline);
  validateSchema(outline, "readableOutline", outlineJson);
  expectNoProtocolEnvelope(outline, outlineJson);
  expect(outline, Array.isArray(outlineJson.entries) && outlineJson.entries.length > 0, "outline returns entries");
  expect(outline, outlineJson.page === null, "outline page is null for normal fixture");
  const normalRef = outlineJson.entries[0].ref;
  setNormalRef(normalRef);
  expect(outline, typeof normalRef === "string" && normalRef.length > 0, "outline exposes a nonempty ref");
  expect(
    outline,
    new Set(outlineJson.entries.map((entry) => entry.ref)).size === outlineJson.entries.length,
    "outline refs are unique"
  );
  expect(outline, outlineJson.entries[0].display.includes("H1"), "outline first entry identifies a top-level heading");

  const read = runCli("read normal readable-json", [
    "read",
    normal,
    "--ref",
    normalRef,
    "--output",
    "readable-json"
  ]);
  expectExit(read, 0);
  expectStderrEmpty(read);
  const readJson = parseJson(read);
  validateSchema(read, "readableRead", readJson);
  expectNoProtocolEnvelope(read, readJson);
  expect(read, readJson.ref === normalRef, "read result preserves ref");
  expect(read, readJson.content.includes("# Guide"), "read content includes heading");
  expect(read, readJson.content.includes("target text"), "read content includes target text");
  expect(read, readJson.content_type === "text/markdown", "read content_type is text/markdown");
  expect(read, readJson.page === null, "read page is null for normal fixture");
  setNormalReadableReadResult(readJson);
}

export function testReadableFindInfo() {
  const normal = fixture("normal.md");

  const find = runCli("find normal readable-json", [
    "find",
    normal,
    "--query",
    "target",
    "--output",
    "readable-json"
  ]);
  expectExit(find, 0);
  expectStderrEmpty(find);
  const findJson = parseJson(find);
  validateSchema(find, "readableFind", findJson);
  expectNoProtocolEnvelope(find, findJson);
  expectNormalFindResult(find, findJson, "readable find");
  setNormalReadableFindResult(findJson);

  const info = runCli("info normal readable-json", [
    "info",
    normal,
    "--output",
    "readable-json"
  ]);
  expectExit(info, 0);
  expectStderrEmpty(info);
  const infoJson = parseJson(info);
  validateSchema(info, "readableInfo", infoJson);
  expectNoProtocolEnvelope(info, infoJson);
  expect(info, infoJson.display.includes("Markdown | text/markdown"), "info readable result has Markdown display");
  for (const capability of ["outline", "read", "find", "info"]) {
    expectIncludes(info, infoJson.capabilities, capability, `info readable includes ${capability} capability`);
  }
}
