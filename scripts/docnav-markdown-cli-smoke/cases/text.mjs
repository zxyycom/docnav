import { fixture, getNormalRef } from "../fixtures.mjs";
import { runCli } from "../runner.mjs";
import {
  expect,
  expectExit,
  expectStderrEmpty,
  expectStdoutIncludes,
  looksLikeJson
} from "../assertions.mjs";

export function testTextOutputs() {
  const normal = fixture("normal.md");
  const ref = getNormalRef();
  const cases = [
    {
      name: "outline normal text",
      args: ["outline", normal, "--output", "text"],
      checks: [
        (record) => expectStdoutIncludes(record, ref),
        (record) => expectStdoutIncludes(record, "Guide"),
        (record) => expectStdoutIncludes(record, "H1"),
        (record) => expectStdoutIncludes(record, "page:")
      ]
    },
    {
      name: "read normal text",
      args: ["read", normal, "--ref", ref, "--output", "text"],
      checks: [
        (record) => expectStdoutIncludes(record, `ref: ${ref}`),
        (record) => expectStdoutIncludes(record, "# Guide"),
        (record) => expectStdoutIncludes(record, "target text"),
        (record) => expectStdoutIncludes(record, "content_type: text/markdown"),
        (record) => expectStdoutIncludes(record, "cost: "),
        (record) => expectStdoutIncludes(record, "page:")
      ]
    },
    {
      name: "find normal text",
      args: ["find", normal, "--query", "target", "--output", "text"],
      checks: [
        (record) => expectStdoutIncludes(record, "[docnav:"),
        (record) => expectStdoutIncludes(record, "target text"),
        (record) => expectStdoutIncludes(record, "target result"),
        (record) =>
          expect(record, (record.stdout.match(/target/g) ?? []).length >= 2, "find text includes both target matches"),
        (record) => expectStdoutIncludes(record, "page:")
      ]
    },
    {
      name: "info normal text",
      args: ["info", normal, "--output", "text"],
      checks: [
        (record) => expectStdoutIncludes(record, "Markdown"),
        (record) => expectStdoutIncludes(record, "text/markdown"),
        (record) => expectStdoutIncludes(record, "capabilities:"),
        (record) => expectStdoutIncludes(record, "outline"),
        (record) => expectStdoutIncludes(record, "read"),
        (record) => expectStdoutIncludes(record, "find"),
        (record) => expectStdoutIncludes(record, "info")
      ]
    }
  ];

  for (const item of cases) {
    const record = runCli(item.name, item.args);
    expectExit(record, 0);
    expectStderrEmpty(record);
    expect(record, !looksLikeJson(record.stdout), "text stdout is not JSON");
    expect(record, !record.stdout.includes("\"protocol_version\""), "text stdout omits protocol envelope");
    for (const check of item.checks) {
      check(record);
    }
  }
}
