import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { resolveBunTestFiles } from "./bun-files.ts";

test("expands Bun test roots with include, ignore and supplemental files", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "docnav-bun-surface-"));
  try {
    writeText(path.join(root, "scripts", "alpha.test.ts"));
    writeText(path.join(root, "scripts", "nested", "beta.test.ts"));
    writeText(path.join(root, "scripts", "ignored", "gamma.test.ts"));
    writeText(path.join(root, "scripts", "not-a-test.ts"));
    writeText(path.join(root, "special", "custom-test.ts"));

    const profile = {
      sourceRoots: ["scripts"],
      include: ["**/*.test.ts"],
      ignore: ["ignored/**"],
      supplementalFiles: ["special/custom-test.ts"]
    };

    assert.deepEqual(resolveBunTestFiles({ workspaceRoot: root, profile }), [
      "scripts/alpha.test.ts",
      "scripts/nested/beta.test.ts",
      "special/custom-test.ts"
    ]);

    writeText(path.join(root, "scripts", "new.test.ts"));
    assert.deepEqual(resolveBunTestFiles({ workspaceRoot: root, profile }), [
      "scripts/alpha.test.ts",
      "scripts/nested/beta.test.ts",
      "scripts/new.test.ts",
      "special/custom-test.ts"
    ]);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

test("rejects invalid, empty and redundant Bun test surfaces", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "docnav-bun-surface-"));
  const outsideRoot = fs.mkdtempSync(path.join(
    os.tmpdir(),
    "docnav-bun-surface-outside-"
  ));
  try {
    writeText(path.join(root, "scripts", "alpha.test.ts"));
    writeText(path.join(outsideRoot, "nested", "external.test.ts"));
    writeText(path.join(outsideRoot, "nested", "supplemental.ts"));
    const base = {
      sourceRoots: ["scripts"],
      include: ["**/*.test.ts"],
      ignore: [],
      supplementalFiles: []
    };

    assert.throws(
      () => resolveBunTestFiles({
        workspaceRoot: root,
        profile: { ...base, sourceRoots: ["missing"] }
      }),
      /source root/
    );
    assert.throws(
      () => resolveBunTestFiles({
        workspaceRoot: root,
        profile: { ...base, include: ["**/*.spec.ts"] }
      }),
      /matched no files/
    );
    assert.throws(
      () => resolveBunTestFiles({
        workspaceRoot: root,
        profile: {
          ...base,
          supplementalFiles: ["scripts/alpha.test.ts"]
        }
      }),
      /already included/
    );
    assert.throws(
      () => resolveBunTestFiles({
        workspaceRoot: root,
        profile: {
          ...base,
          supplementalFiles: ["missing.test.ts"]
        }
      }),
      /supplemental file/
    );
    for (const field of ["include", "ignore"] as const) {
      for (const pattern of ["!ignored/**", "#ignored/**"]) {
        assert.throws(
          () => resolveBunTestFiles({
            workspaceRoot: root,
            profile: { ...base, [field]: [pattern] }
          }),
          /positive relative POSIX globs/
        );
      }
    }

    const linkedRoot = path.join(root, "linked");
    fs.symlinkSync(
      outsideRoot,
      linkedRoot,
      process.platform === "win32" ? "junction" : "dir"
    );
    assert.throws(
      () => resolveBunTestFiles({
        workspaceRoot: root,
        profile: { ...base, sourceRoots: ["linked/nested"] }
      }),
      /symbolic link/
    );
    assert.throws(
      () => resolveBunTestFiles({
        workspaceRoot: root,
        profile: {
          ...base,
          supplementalFiles: ["linked/nested/supplemental.ts"]
        }
      }),
      /symbolic link/
    );
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
    fs.rmSync(outsideRoot, { force: true, recursive: true });
  }
});

function writeText(sourcePath: string): void {
  fs.mkdirSync(path.dirname(sourcePath), { recursive: true });
  fs.writeFileSync(sourcePath, "test content", "utf8");
}
