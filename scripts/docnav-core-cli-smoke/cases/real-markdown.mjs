import { createProject, createRealMarkdownAdapter, writeRegistry } from "../fixtures.mjs";
import { runCli } from "../harness.mjs";
import {
  expect,
  expectExit,
  expectNoProtocolEnvelope,
  expectStderrEmpty,
  parseJson
} from "../assertions.mjs";
import { validateSchema } from "../harness.mjs";

export function testRealMarkdownOutlineRefRead() {
  const project = createProject("real-markdown-outline-read");
  const markdown = createRealMarkdownAdapter(project);
  writeRegistry(project, [markdown]);

  const outline = runCli("core outline real markdown readable-json", [
    "outline",
    project.normalRelPath,
    "--output",
    "readable-json"
  ], { project });
  expectExit(outline, 0);
  expectStderrEmpty(outline);
  const outlineJson = parseJson(outline);
  validateSchema(outline, "readableOutline", outlineJson);
  expectNoProtocolEnvelope(outline, outlineJson);
  expect(outline, Array.isArray(outlineJson.entries) && outlineJson.entries.length > 0, "outline returns entries");
  const ref = outlineJson.entries[0].ref;
  expect(outline, typeof ref === "string" && ref.length > 0, "outline exposes a nonempty ref");

  const read = runCli("core read real markdown readable-json", [
    "read",
    project.normalRelPath,
    "--ref",
    ref,
    "--output",
    "readable-json"
  ], { project });
  expectExit(read, 0);
  expectStderrEmpty(read);
  const readJson = parseJson(read);
  validateSchema(read, "readableRead", readJson);
  expectNoProtocolEnvelope(read, readJson);
  expect(read, readJson.ref === ref, "read preserves adapter ref");
  expect(read, readJson.content.includes("# Guide"), "read content includes Markdown heading");
  expect(read, readJson.content.includes("target text"), "read content includes fixture body");
  expect(read, readJson.content_type === "text/markdown", "read preserves content_type");
}
