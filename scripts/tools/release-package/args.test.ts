import assert from "node:assert/strict";
import test from "node:test";

import { parseManifestArgs, parseOptionalTarget } from "./args.ts";

test("package selection defaults to the current host package", () => {
  assert.deepEqual(parseManifestArgs([]), {
    manifestPath: null,
    target: null,
    expectProducerKind: null,
    expectSourceDirty: null,
  });
});

test("package selection accepts a target", () => {
  assert.equal(
    parseManifestArgs(["--target", "x86_64-pc-windows-msvc"]).target,
    "x86_64-pc-windows-msvc",
  );
});

test("package selection keeps explicit manifest support", () => {
  assert.equal(
    parseManifestArgs(["--manifest", "download/package/manifest.json"])
      .manifestPath,
    "download/package/manifest.json",
  );
});

test("package selection rejects ambiguous selectors", () => {
  assert.throws(
    () =>
      parseManifestArgs([
        "--manifest",
        "package/manifest.json",
        "--target",
        "x86_64-pc-windows-msvc",
      ]),
    /cannot be used together/,
  );
});

test("package selection rejects target paths", () => {
  assert.throws(
    () => parseManifestArgs(["--target", "artifacts/package"]),
    /Rust target triple/,
  );
});

test("package build target defaults to host target", () => {
  assert.equal(parseOptionalTarget([]), null);
});

test("package build target accepts one target option", () => {
  assert.equal(parseOptionalTarget(["--target", "x86_64-ExternalValue-linux-gnu"]), "x86_64-ExternalValue-linux-gnu");
});

test("package build target rejects extra options and paths", () => {
  assert.throws(() => parseOptionalTarget(["--manifest", "package/manifest.json"]), /ExternalValue option --manifest/);
  assert.throws(() => parseOptionalTarget(["--target", "artifacts/package"]), /Rust target triple/);
  assert.throws(() => parseOptionalTarget(["x86_64-ExternalValue-linux-gnu"]), /unexpected positional argument/);
});
