import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  validateReleaseCandidate,
  type CandidateExpectations,
} from "./candidate.ts";
import type { ReleaseManifest } from "./config.ts";
import { stagePublicFiles } from "./public.ts";

const version = "0.1.0-beta.1";
const gitCommit = "0123456789abcdef0123456789abcdef01234567";
const supportedTargets = [
  "x86_64-unknown-linux-gnu",
  "x86_64-pc-windows-msvc",
] as const;
type SupportedTarget = (typeof supportedTargets)[number];

// @case AUX-RELEASE-CANDIDATE-001
test("accepts an exact manual-run candidate without modifying its files", () => {
  using fixture = createCandidateFixture();
  const before = snapshotTree(fixture.candidateRoot);

  const result = validateReleaseCandidate(
    fixture.candidateRoot,
    expectations(),
  );

  assert.equal(result.version, version);
  assert.equal(result.gitCommit, gitCommit);
  assert.deepEqual(
    result.targets.map(({ target }) => target),
    supportedTargets,
  );
  assert.deepEqual(snapshotTree(fixture.candidateRoot), before);
});

test("accepts only the matching workspace tag and tag commit", () => {
  using fixture = createCandidateFixture();

  validateReleaseCandidate(
    fixture.candidateRoot,
    expectations({ name: `v${version}`, gitCommit }),
  );
  assert.throws(
    () =>
      validateReleaseCandidate(
        fixture.candidateRoot,
        expectations({ name: "v0.1.0-beta.2", gitCommit }),
      ),
    /candidate tag must be v0\.1\.0-beta\.1/,
  );
  assert.throws(
    () =>
      validateReleaseCandidate(
        fixture.candidateRoot,
        expectations({
          name: `v${version}`,
          gitCommit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        }),
      ),
    /tag commit must match candidate commit/,
  );
});

test("rejects a candidate with a non-exact direct target set", () => {
  using fixture = createCandidateFixture();
  fs.mkdirSync(path.join(fixture.candidateRoot, "aarch64-unknown-linux-gnu"));

  assert.throws(
    () => validateReleaseCandidate(fixture.candidateRoot, expectations()),
    /candidate must contain exactly the supported target directories/,
  );
});

test("rejects a target with a non-exact public file set", () => {
  using fixture = createCandidateFixture();
  fs.writeFileSync(
    path.join(
      fixture.targets["x86_64-unknown-linux-gnu"].publicDir,
      "unexpected.txt",
    ),
    "unexpected",
  );

  assert.throws(
    () => validateReleaseCandidate(fixture.candidateRoot, expectations()),
    /public directory must contain exactly the binary and checksum/,
  );
});

test("rejects workspace version and manifest commit mismatches", () => {
  using fixture = createCandidateFixture();

  assert.throws(
    () =>
      validateReleaseCandidate(fixture.candidateRoot, {
        ...expectations(),
        version: "0.1.0-beta.2",
      }),
    /candidate version root must be v0\.1\.0-beta\.2/,
  );

  updateManifest(
    fixture.targets["x86_64-pc-windows-msvc"],
    (manifest) => {
      manifest.git_commit = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    },
  );
  assert.throws(
    () => validateReleaseCandidate(fixture.candidateRoot, expectations()),
    /manifest\.git_commit must match candidate commit/,
  );
});

test("rejects dirty checkout or manifest evidence", () => {
  using fixture = createCandidateFixture();

  assert.throws(
    () =>
      validateReleaseCandidate(fixture.candidateRoot, {
        ...expectations(),
        sourceDirty: true,
      }),
    /workspace source must be clean/,
  );

  updateManifest(
    fixture.targets["x86_64-unknown-linux-gnu"],
    (manifest) => {
      manifest.source_dirty = true;
    },
  );
  assert.throws(
    () => validateReleaseCandidate(fixture.candidateRoot, expectations()),
    /manifest\.source_dirty must be false/,
  );
});

test("rejects package evidence from a different workflow run", () => {
  using fixture = createCandidateFixture();
  updateManifest(
    fixture.targets["x86_64-unknown-linux-gnu"],
    (manifest) => {
      manifest.producer = {
        kind: "github-actions",
        workflow: "Release package",
        run_id: 99,
        run_attempt: 3,
      };
    },
  );

  assert.throws(
    () => validateReleaseCandidate(fixture.candidateRoot, expectations()),
    /manifest producer must match current workflow run/,
  );
});

test("rejects canonical package and public hash mismatches", () => {
  {
    using fixture = createCandidateFixture();
    const packageBinaryPath =
      fixture.targets["x86_64-unknown-linux-gnu"].packageBinaryPath;
    const tamperedPackage = fs.readFileSync(packageBinaryPath);
    tamperedPackage[0] = tamperedPackage[0] === 0 ? 1 : 0;
    fs.writeFileSync(packageBinaryPath, tamperedPackage);
    assert.throws(
      () => validateReleaseCandidate(fixture.candidateRoot, expectations()),
      /sha256/,
    );
  }

  {
    using fixture = createCandidateFixture();
    const target = fixture.targets["x86_64-pc-windows-msvc"];
    fs.writeFileSync(target.publicBinaryPath, "tampered-public");
    const publicHash = sha256File(target.publicBinaryPath);
    fs.writeFileSync(
      target.publicChecksumPath,
      `${publicHash}  ${path.basename(target.publicBinaryPath)}\n`,
    );
    assert.throws(
      () => validateReleaseCandidate(fixture.candidateRoot, expectations()),
      /public binary sha256 must match canonical package binary/,
    );
  }

  {
    using fixture = createCandidateFixture();
    const target = fixture.targets["x86_64-unknown-linux-gnu"];
    fs.writeFileSync(
      target.publicChecksumPath,
      `${"0".repeat(64)}  ${path.basename(target.publicBinaryPath)}\n`,
    );
    assert.throws(
      () => validateReleaseCandidate(fixture.candidateRoot, expectations()),
      /public checksum must match binary hash and filename/,
    );
  }
});

type TargetFixture = {
  manifestPath: string;
  packageBinaryPath: string;
  publicBinaryPath: string;
  publicChecksumPath: string;
  publicDir: string;
};

type CandidateFixture = {
  [Symbol.dispose](): void;
  candidateRoot: string;
  targets: Record<SupportedTarget, TargetFixture>;
};

function expectations(
  tag: CandidateExpectations["tag"] = null,
): CandidateExpectations {
  return {
    gitCommit,
    producer: {
      kind: "github-actions",
      workflow: "Release package",
      run_id: 42,
      run_attempt: 3,
    },
    sourceDirty: false,
    tag,
    version,
  };
}

function createCandidateFixture(): CandidateFixture {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "docnav-candidate-"));
  const candidateRoot = path.join(
    tempDir,
    "artifacts",
    "docnav",
    `v${version}`,
  );
  const linux = createTargetFixture(
    candidateRoot,
    "x86_64-unknown-linux-gnu",
  );
  const windows = createTargetFixture(
    candidateRoot,
    "x86_64-pc-windows-msvc",
  );

  return {
    [Symbol.dispose]() {
      fs.rmSync(tempDir, { recursive: true, force: true });
    },
    candidateRoot,
    targets: {
      "x86_64-unknown-linux-gnu": linux,
      "x86_64-pc-windows-msvc": windows,
    },
  };
}

function createTargetFixture(
  candidateRoot: string,
  target: SupportedTarget,
): TargetFixture {
  const packageDir = path.join(candidateRoot, target, "package");
  const packageBinaryName = target.includes("windows")
    ? "docnav.exe"
    : "docnav";
  const packageBinaryPath = path.join(packageDir, packageBinaryName);
  const manifestPath = path.join(packageDir, "manifest.json");
  const packageBytes = Buffer.from(`package-${target}`);

  fs.mkdirSync(packageDir, { recursive: true });
  fs.writeFileSync(packageBinaryPath, packageBytes);
  fs.writeFileSync(
    manifestPath,
    `${JSON.stringify(
      {
        schema_version: 1,
        product: "docnav",
        version,
        target,
        generated_at: "2026-07-20T00:00:00.000Z",
        git_commit: gitCommit,
        source_dirty: false,
        producer: {
          kind: "github-actions",
          workflow: "Release package",
          run_id: 42,
          run_attempt: 3,
        },
        files: [
          {
            path: packageBinaryName,
            component: "core",
            size_bytes: packageBytes.length,
            sha256: sha256(packageBytes),
          },
        ],
      } satisfies ReleaseManifest,
      null,
      2,
    )}\n`,
  );

  const fixture = {
    manifestPath,
    packageBinaryPath,
    publicBinaryPath: "",
    publicChecksumPath: "",
    publicDir: "",
  };
  writePackageChecksums(fixture);
  const staged = stagePublicFiles(manifestPath, {
    expectProducerKind: "github-actions",
    expectSourceDirty: false,
  });

  return {
    ...fixture,
    publicBinaryPath: staged.publicBinaryPath,
    publicChecksumPath: staged.checksumPath,
    publicDir: staged.publicDir,
  };
}

function updateManifest(
  fixture: TargetFixture,
  update: (manifest: ReleaseManifest) => void,
): void {
  const manifest = JSON.parse(
    fs.readFileSync(fixture.manifestPath, "utf8"),
  ) as ReleaseManifest;
  update(manifest);
  fs.writeFileSync(
    fixture.manifestPath,
    `${JSON.stringify(manifest, null, 2)}\n`,
  );
  writePackageChecksums(fixture);
}

function writePackageChecksums(
  fixture: Pick<TargetFixture, "manifestPath" | "packageBinaryPath">,
): void {
  const packageBinaryName = path.basename(fixture.packageBinaryPath);
  const entries = [
    [packageBinaryName, sha256File(fixture.packageBinaryPath)],
    ["manifest.json", sha256File(fixture.manifestPath)],
  ]
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([fileName, hash]) => `${hash}  ${fileName}`);
  fs.writeFileSync(
    path.join(path.dirname(fixture.manifestPath), "SHA256SUMS.txt"),
    `${entries.join("\n")}\n`,
  );
}

function snapshotTree(root: string): string[] {
  const snapshot: string[] = [];
  visit(root, "");
  return snapshot;

  function visit(directory: string, relativeDirectory: string): void {
    for (const entry of fs
      .readdirSync(directory, { withFileTypes: true })
      .sort((left, right) => left.name.localeCompare(right.name))) {
      const relativePath = path.join(relativeDirectory, entry.name);
      const entryPath = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        snapshot.push(`dir:${relativePath}`);
        visit(entryPath, relativePath);
      } else {
        snapshot.push(`file:${relativePath}:${sha256File(entryPath)}`);
      }
    }
  }
}

function sha256File(filePath: string): string {
  return sha256(fs.readFileSync(filePath));
}

function sha256(content: Buffer | string): string {
  return crypto.createHash("sha256").update(content).digest("hex");
}
