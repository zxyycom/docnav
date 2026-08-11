import { describe, expect, test } from "bun:test";
import { mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { findMarkdownLinkFailures } from "./links.ts";

describe("markdown link validation", () => {
  test("accepts local, inline-code, duplicate and encoded-path fragments", () => {
    const root = fixtureRoot();
    const source = join(root, "source.md");
    const target = join(root, "target.md");
    const encodedTarget = join(root, "target#.md");
    try {
      writeFileSync(source, [
        "# Source",
        "## Local section",
        "[local](#local-section)",
        "[inline](target.md#api-surface)",
        "[duplicate](target.md#repeated-1)",
        "[encoded-path](target%23.md#present)",
        ""
      ].join("\n"), "utf8");
      writeFileSync(target, [
        "# API `surface`",
        "## Repeated",
        "## Repeated",
        ""
      ].join("\n"), "utf8");
      writeFileSync(encodedTarget, "# Present\n", "utf8");

      expect(findMarkdownLinkFailures(
        [source, target, encodedTarget],
        root
      )).toEqual([]);
    } finally {
      rmSync(root, { force: true, recursive: true });
    }
  });

  test("rejects missing fragments and ignores heading-looking text in fenced code", () => {
    const root = fixtureRoot();
    const source = join(root, "source.md");
    const target = join(root, "target.md");
    try {
      writeFileSync(source, [
        "# Source",
        "[missing](target.md#missing-heading)",
        "[fenced](target.md#not-a-heading)",
        ""
      ].join("\n"), "utf8");
      writeFileSync(target, [
        "# Target",
        "```markdown",
        "## Not a heading",
        "```",
        ""
      ].join("\n"), "utf8");

      expect(findMarkdownLinkFailures([source, target], root)).toEqual([
        { sourcePath: source, target: "target.md#missing-heading", reason: "missing_fragment" },
        { sourcePath: source, target: "target.md#not-a-heading", reason: "missing_fragment" }
      ]);
    } finally {
      rmSync(root, { force: true, recursive: true });
    }
  });

  test("rejects paths outside the validation root and checks .markdown fragments", () => {
    const root = fixtureRoot();
    const source = join(root, "source.md");
    const target = join(root, "guide.markdown");
    try {
      writeFileSync(source, [
        "# Source",
        "[escaped](../../outside.md)",
        "[markdown](guide.markdown#missing)",
        ""
      ].join("\n"), "utf8");
      writeFileSync(target, "# Present\n", "utf8");

      expect(findMarkdownLinkFailures([source, target], root)).toEqual([
        { sourcePath: source, target: "../../outside.md", reason: "outside_root" },
        { sourcePath: source, target: "guide.markdown#missing", reason: "missing_fragment" }
      ]);

      if (process.platform !== "win32") {
        const outsideRoot = fixtureRoot();
        const outsideSource = join(outsideRoot, "outside.md");
        const linkedTarget = join(root, "linked-target.md");
        const symlinkLinkSource = join(root, "symlink-source.md");
        try {
          writeFileSync(outsideSource, "# Outside\n", "utf8");
          symlinkSync(outsideSource, linkedTarget, "file");
          writeFileSync(symlinkLinkSource, "[outside](linked-target.md)\n", "utf8");
          expect(findMarkdownLinkFailures([symlinkLinkSource], root)).toEqual([{
            sourcePath: symlinkLinkSource,
            target: "linked-target.md",
            reason: "outside_root"
          }]);
        } finally {
          rmSync(outsideRoot, { force: true, recursive: true });
        }
      }
    } finally {
      rmSync(root, { force: true, recursive: true });
    }
  });
});

function fixtureRoot(): string {
  return mkdtempSync(join(tmpdir(), "docnav-links-"));
}
