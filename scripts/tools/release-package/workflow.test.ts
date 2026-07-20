import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";

const workflow = fs.readFileSync(
  new URL("../../../.github/workflows/release-package.yml", import.meta.url),
  "utf8",
);

// @case AUX-RELEASE-WORKFLOW-001
test("release workflow keeps manual validation and gates promotion on Beta tags", () => {
  const workflowHeader = before(workflow, "\njobs:\n");

  assert.match(workflowHeader, /\n {2}workflow_dispatch:\s*\n/);
  assert.match(
    workflowHeader,
    /\n {2}push:\n {4}tags:\n {6}- ["']v\*-beta\.\*["']\n/,
  );
  assert.match(workflowHeader, /\npermissions:\n {2}contents: read\n/);
  assert.equal(occurrences(workflow, "contents: write"), 1);
});

test("native matrix stages one exact package and public artifact per supported target", () => {
  const packageJob = job("package", "aggregate");
  const targets = [...packageJob.matchAll(/^\s+target: (\S+)$/gm)].map(
    (match) => match[1],
  );

  assert.deepEqual(targets, [
    "x86_64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
  ]);
  assert.match(
    packageJob,
    /runs-on: ubuntu-latest\n\s+target: x86_64-unknown-linux-gnu/,
  );
  assert.match(
    packageJob,
    /runs-on: windows-latest\n\s+target: x86_64-pc-windows-msvc/,
  );

  assertInOrder(packageJob, [
    "bun run package:docnav",
    "bun run verify:docnav-package",
    "bun run smoke:docnav-package",
    "bun run stage:docnav-public",
    "actions/upload-artifact@",
  ]);
  for (const command of [
    "bun run verify:docnav-package",
    "bun run smoke:docnav-package",
    "bun run stage:docnav-public",
  ]) {
    const line = lineContaining(packageJob, command);
    assert.match(line, /--manifest "\$\{\{ steps\.info\.outputs\.manifest_path \}\}"/);
    assert.match(line, /--expect-producer-kind github-actions/);
    assert.match(line, /--expect-source-dirty false/);
  }

  const upload = after(packageJob, "actions/upload-artifact@");
  assert.match(upload, /name: \$\{\{ matrix\.target \}\}/);
  assert.match(upload, /path: \|/);
  assert.match(upload, /\$\{\{ steps\.info\.outputs\.package_dir \}\}/);
  assert.match(
    upload,
    /artifacts\/docnav\/v\$\{\{ steps\.info\.outputs\.version \}\}\/\$\{\{ matrix\.target \}\}\/public/,
  );
  assert.doesNotMatch(upload, /^\s+target\//m);
});

test("aggregate validation consumes current-run artifacts for manual and tag inputs", () => {
  const aggregateJob = job("aggregate", "publish");

  assert.match(aggregateJob, /\n {4}needs: package\n/);
  assert.match(
    aggregateJob,
    /\n {4}permissions:\n {6}contents: read\n/,
  );
  assert.match(aggregateJob, /actions\/download-artifact@/);
  assert.match(aggregateJob, /pattern: ["']x86_64-\*["']/);
  assert.match(
    aggregateJob,
    /path: artifacts\/docnav\/v\$\{\{ steps\.info\.outputs\.version \}\}/,
  );
  assert.doesNotMatch(aggregateJob, /\n\s+run-id:/);

  const manualValidation = step(
    aggregateJob,
    "Validate manual release candidate",
    "Validate tagged release candidate",
  );
  assert.match(
    manualValidation,
    /if: github\.event_name == 'workflow_dispatch'/,
  );
  assert.match(manualValidation, /bun run validate:docnav-candidate/);
  assert.doesNotMatch(manualValidation, /\s--tag(?:\s|$)/);

  const tagValidation = step(
    aggregateJob,
    "Validate tagged release candidate",
  );
  assert.match(tagValidation, /if: github\.event_name == 'push'/);
  assert.match(
    tagValidation,
    /\n {8}env:\n {10}RELEASE_TAG: \$\{\{ github\.ref_name \}\}\n/,
  );
  const tagValidationRun = after(tagValidation, "\n        run:");
  assert.match(tagValidationRun, /bun run validate:docnav-candidate/);
  assert.match(tagValidationRun, /--tag "\$env:RELEASE_TAG"/);
  assert.doesNotMatch(tagValidationRun, /\$\{\{ github\.ref_name \}\}/);
});

test("publish is the single writer and creates one new prerelease from four public files", () => {
  const publishJob = job("publish");

  assert.match(publishJob, /\n {4}needs: aggregate\n/);
  assert.match(
    publishJob,
    /if: github\.event_name == 'push' && startsWith\(github\.ref, 'refs\/tags\/'\)/,
  );
  assert.match(
    publishJob,
    /\n {4}permissions:\n {6}contents: write\n/,
  );
  assert.match(publishJob, /actions\/download-artifact@/);
  assert.match(publishJob, /pattern: ["']x86_64-\*["']/);
  assert.doesNotMatch(publishJob, /\n\s+run-id:/);

  const createRelease = step(publishJob, "Create GitHub prerelease");
  assert.match(
    createRelease,
    /\n {10}RELEASE_TAG: \$\{\{ github\.ref_name \}\}\n/,
  );
  const createReleaseRun = after(createRelease, "\n        run: |");
  assert.match(createReleaseRun, /\$tag = \$env:RELEASE_TAG/);
  assert.doesNotMatch(createReleaseRun, /\$\{\{ github\.ref_name \}\}/);

  assertInOrder(publishJob, ["gh release view", "gh release create"]);
  const beforeCreate = before(publishJob, "gh release create");
  assert.match(beforeCreate, /\$releaseLookupExit = \$LASTEXITCODE/);
  assert.match(beforeCreate, /if \(\$releaseLookupExit -eq 0\)/);
  assert.match(beforeCreate, /throw "release \$tag already exists"/);
  assert.equal(occurrences(publishJob, "gh release create"), 1);
  assert.match(publishJob, /--verify-tag/);
  assert.match(publishJob, /--prerelease/);
  assert.match(publishJob, /--latest=false/);
  assert.match(publishJob, /--notes-file "docs\/releases\/\$tag\.md"/);

  const publicAssets = publishJob
    .split(/\r?\n/)
    .filter((line) => /"\$candidateRoot\/[^"]+\/public\/[^"]+"/.test(line));
  assert.deepEqual(publicAssets, [
    '            "$candidateRoot/x86_64-unknown-linux-gnu/public/docnav-v$version-x86_64-unknown-linux-gnu" `',
    '            "$candidateRoot/x86_64-unknown-linux-gnu/public/docnav-v$version-x86_64-unknown-linux-gnu.sha256" `',
    '            "$candidateRoot/x86_64-pc-windows-msvc/public/docnav-v$version-x86_64-pc-windows-msvc.exe" `',
    '            "$candidateRoot/x86_64-pc-windows-msvc/public/docnav-v$version-x86_64-pc-windows-msvc.exe.sha256" `',
  ]);

  assert.doesNotMatch(
    publishJob,
    /gh release (?:upload|edit)|--clobber|continue-on-error/,
  );
  const afterCreate = after(publishJob, "gh release create");
  assert.match(afterCreate, /\$LASTEXITCODE -ne 0/);
  assert.match(afterCreate, /throw "failed to create release \$tag"/);
});

function job(name: string, nextName?: string): string {
  const start = `\n  ${name}:\n`;
  const end = nextName ? `\n  ${nextName}:\n` : undefined;
  return slice(workflow, start, end);
}

function step(source: string, name: string, nextName?: string): string {
  const start = `- name: ${name}`;
  const end = nextName ? `- name: ${nextName}` : undefined;
  return slice(source, start, end);
}

function slice(source: string, start: string, end?: string): string {
  const startIndex = source.indexOf(start);
  assert.notEqual(startIndex, -1, `missing ${start}`);
  const contentStart = startIndex + start.length;
  if (!end) {
    return source.slice(contentStart);
  }
  const endIndex = source.indexOf(end, contentStart);
  assert.notEqual(endIndex, -1, `missing ${end}`);
  return source.slice(contentStart, endIndex);
}

function before(source: string, marker: string): string {
  const index = source.indexOf(marker);
  assert.notEqual(index, -1, `missing ${marker}`);
  return source.slice(0, index);
}

function after(source: string, marker: string): string {
  const index = source.indexOf(marker);
  assert.notEqual(index, -1, `missing ${marker}`);
  return source.slice(index + marker.length);
}

function lineContaining(source: string, marker: string): string {
  const line = source.split(/\r?\n/).find((candidate) => candidate.includes(marker));
  assert.ok(line, `missing line containing ${marker}`);
  return line;
}

function occurrences(source: string, marker: string): number {
  return source.split(marker).length - 1;
}

function assertInOrder(source: string, markers: string[]): void {
  let previous = -1;
  for (const marker of markers) {
    const index = source.indexOf(marker);
    assert.ok(index > previous, `${marker} must follow the previous workflow gate`);
    previous = index;
  }
}
