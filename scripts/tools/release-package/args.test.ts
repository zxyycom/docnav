import assert from "node:assert/strict";
import path from "node:path";
import test from "node:test";

import { parseManifestArgs, parseOptionalTarget } from "./args.ts";
import { artifactsRoot, resolvePackageLayout } from "./config.ts";

test("package selection parses supported selectors", () => {
  const cases = [
    { args: [], expected: { manifestPath: null, target: null } },
    {
      args: ["--target", "x86_64-pc-windows-msvc"],
      expected: { manifestPath: null, target: "x86_64-pc-windows-msvc" },
    },
    {
      args: ["--manifest", "download/package/manifest.json"],
      expected: {
        manifestPath: "download/package/manifest.json",
        target: null,
      },
    },
  ] as const;

  for (const { args, expected } of cases) {
    const parsed = parseManifestArgs([...args]);
    assert.deepEqual(
      { manifestPath: parsed.manifestPath, target: parsed.target },
      expected,
    );
  }
});

test("package selection rejects invalid selectors", () => {
  const cases = [
    {
      args: [
        "--manifest",
        "package/manifest.json",
        "--target",
        "x86_64-pc-windows-msvc",
      ],
      diagnostic: /cannot be used together/,
    },
    {
      args: ["--target", "artifacts/package"],
      diagnostic: /Rust target triple/,
    },
  ];

  for (const { args, diagnostic } of cases) {
    assert.throws(() => parseManifestArgs(args), diagnostic);
  }
});

test("package build target parses supported selectors", () => {
  const cases = [
    { args: [], expected: null },
    {
      args: ["--target", "x86_64-unknown-linux-gnu"],
      expected: "x86_64-unknown-linux-gnu",
    },
    {
      args: ["--target", "thumbv8m.main-none-eabi"],
      expected: "thumbv8m.main-none-eabi",
    },
  ] as const;

  for (const { args, expected } of cases) {
    assert.equal(parseOptionalTarget([...args]), expected);
  }
});

test("package build target rejects invalid selectors", () => {
  const cases = [
    {
      args: ["--manifest", "package/manifest.json"],
      diagnostic: /unknown option --manifest/,
    },
    {
      args: ["--target", "artifacts/package"],
      diagnostic: /Rust target triple/,
    },
    {
      args: ["x86_64-unknown-linux-gnu"],
      diagnostic: /unexpected positional argument/,
    },
  ];

  for (const { args, diagnostic } of cases) {
    assert.throws(() => parseOptionalTarget(args), diagnostic);
  }
});

test("package layout keeps every cleanup root under its version root", () => {
  const layout = resolvePackageLayout("0.1.0-beta.1", "thumbv8m.main-none-eabi");
  const versionRoot = path.join(artifactsRoot, "v0.1.0-beta.1");

  assert.equal(path.dirname(layout.releaseRoot), versionRoot);
  assert.throws(
    () => resolvePackageLayout("0.1.0-beta.1", "."),
    /release target root must be a strict child of the version root/,
  );
  assert.throws(
    () => resolvePackageLayout("../..", "x86_64-unknown-linux-gnu"),
    /release version root must be a strict child of the artifacts root/,
  );
});
