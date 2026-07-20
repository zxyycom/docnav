import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { stagePublicFiles } from "./public.ts";

const version = "0.1.0-beta.1";

// @case AUX-RELEASE-PUBLIC-001
test("stages the exact Linux public file set from canonical package evidence", () => {
  using fixture = createPackageFixture("x86_64-unknown-linux-gnu", Buffer.from("linux-binary"));

  const result = stagePublicFiles(fixture.manifestPath);
  const publicFileName = "docnav-v0.1.0-beta.1-x86_64-unknown-linux-gnu";

  assert.deepEqual(fs.readdirSync(result.publicDir).sort(), [
    publicFileName,
    `${publicFileName}.sha256`,
  ]);
  assert.deepEqual(fs.readFileSync(result.publicBinaryPath), fixture.binaryBytes);
  assert.equal(
    fs.readFileSync(result.checksumPath, "utf8"),
    `${fixture.binaryHash}  ${publicFileName}\n`,
  );
});

test("stages the exact Windows public file set from canonical package evidence", () => {
  using fixture = createPackageFixture("x86_64-pc-windows-msvc", Buffer.from("windows-binary"));

  const result = stagePublicFiles(fixture.manifestPath);
  const publicFileName = "docnav-v0.1.0-beta.1-x86_64-pc-windows-msvc.exe";

  assert.deepEqual(fs.readdirSync(result.publicDir).sort(), [
    publicFileName,
    `${publicFileName}.sha256`,
  ]);
  assert.deepEqual(fs.readFileSync(result.publicBinaryPath), fixture.binaryBytes);
  assert.equal(
    fs.readFileSync(result.checksumPath, "utf8"),
    `${fixture.binaryHash}  ${publicFileName}\n`,
  );
});

test("missing canonical package evidence fails without modifying an existing public set", () => {
  using fixture = createPackageFixture("x86_64-unknown-linux-gnu", Buffer.from("linux-binary"));
  const staged = stagePublicFiles(fixture.manifestPath);
  fs.rmSync(fixture.binaryPath);

  assert.throws(() => stagePublicFiles(fixture.manifestPath));
  assert.deepEqual(fs.readFileSync(staged.publicBinaryPath), fixture.binaryBytes);
});

test("mismatched canonical package evidence fails without modifying an existing public set", () => {
  using fixture = createPackageFixture("x86_64-unknown-linux-gnu", Buffer.from("linux-binary"));
  const staged = stagePublicFiles(fixture.manifestPath);
  fs.writeFileSync(fixture.binaryPath, Buffer.from("linux-tamper"));

  assert.throws(
    () => stagePublicFiles(fixture.manifestPath),
    /sha256 must match actual file hash/,
  );
  assert.deepEqual(fs.readFileSync(staged.publicBinaryPath), fixture.binaryBytes);
});

test("a checksum write failure removes public files created after validation", () => {
  using fixture = createPackageFixture("x86_64-unknown-linux-gnu", Buffer.from("linux-binary"));
  const originalWriteFileSync = fs.writeFileSync;
  fs.writeFileSync = (
    (...args: Parameters<typeof fs.writeFileSync>) => {
      if (String(args[0]).endsWith(".sha256")) {
        throw new Error("injected checksum write failure");
      }
      Reflect.apply(originalWriteFileSync, fs, args);
    }
  ) as typeof fs.writeFileSync;

  try {
    assert.throws(
      () => stagePublicFiles(fixture.manifestPath),
      /injected checksum write failure/,
    );
  } finally {
    fs.writeFileSync = originalWriteFileSync;
  }
  assert.equal(
    fs.existsSync(
      path.join(path.dirname(path.dirname(fixture.manifestPath)), "public"),
    ),
    false,
  );
});

test("a missing manifest does not remove an unrelated public directory", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "docnav-public-invalid-"));
  const manifestPath = path.join(tempDir, "package", "manifest.json");
  const markerPath = path.join(tempDir, "public", "marker.txt");
  fs.mkdirSync(path.dirname(markerPath), { recursive: true });
  fs.writeFileSync(markerPath, "keep");

  try {
    assert.throws(() => stagePublicFiles(manifestPath));
    assert.equal(fs.readFileSync(markerPath, "utf8"), "keep");
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

type PackageFixture = {
  [Symbol.dispose](): void;
  binaryBytes: Buffer;
  binaryHash: string;
  binaryPath: string;
  manifestPath: string;
};

function createPackageFixture(target: string, binaryBytes: Buffer): PackageFixture {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "docnav-public-"));
  const packageDir = path.join(
    tempDir,
    "artifacts",
    "docnav",
    `v${version}`,
    target,
    "package",
  );
  const binaryName = target.includes("windows") ? "docnav.exe" : "docnav";
  const binaryPath = path.join(packageDir, binaryName);
  const manifestPath = path.join(packageDir, "manifest.json");
  const checksumsPath = path.join(packageDir, "SHA256SUMS.txt");
  const binaryHash = sha256(binaryBytes);

  fs.mkdirSync(packageDir, { recursive: true });
  fs.writeFileSync(binaryPath, binaryBytes);
  fs.writeFileSync(
    manifestPath,
    `${JSON.stringify(
      {
        schema_version: 1,
        product: "docnav",
        version,
        target,
        generated_at: "2026-07-20T00:00:00.000Z",
        git_commit: "0123456789abcdef",
        source_dirty: false,
        producer: {
          kind: "local",
          workflow: null,
          run_id: null,
          run_attempt: null,
        },
        files: [
          {
            path: binaryName,
            component: "core",
            size_bytes: binaryBytes.length,
            sha256: binaryHash,
          },
        ],
      },
      null,
      2,
    )}\n`,
  );
  fs.writeFileSync(
    checksumsPath,
    `${binaryHash}  ${binaryName}\n${sha256(fs.readFileSync(manifestPath))}  manifest.json\n`,
  );

  return {
    [Symbol.dispose]() {
      fs.rmSync(tempDir, { recursive: true, force: true });
    },
    binaryBytes,
    binaryHash,
    binaryPath,
    manifestPath,
  };
}

function sha256(content: Buffer): string {
  return crypto.createHash("sha256").update(content).digest("hex");
}
