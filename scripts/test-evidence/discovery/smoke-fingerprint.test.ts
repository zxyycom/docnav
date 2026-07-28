import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { createSmokeSourceFingerprint } from "./smoke-fingerprint.ts";

test("fingerprints the reachable smoke run implementation without unrelated declarations", () => {
  const root = fs.mkdtempSync(path.join(
    os.tmpdir(),
    "docnav-smoke-fingerprint-"
  ));
  try {
    writeText(path.join(root, "cases", "tasks.ts"), [
      "import { runCase } from \"./run-case.ts\";",
      "",
      "export function createTasks() {",
      "  return [{ id: \"CASE-001\", label: \"case\", run: runCase }];",
      "}",
      ""
    ].join("\n"));
    writeRunCase(root, "unchanged");
    writeAssertion(root, "first");

    const initial = fingerprint(root);
    writeRunCase(root, "changed but unreachable");
    const unrelatedChange = fingerprint(root);
    assert.equal(unrelatedChange, initial);

    writeAssertion(root, "second");
    assert.notEqual(fingerprint(root), initial);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

function fingerprint(workspaceRoot: string): string {
  return createSmokeSourceFingerprint({
    workspaceRoot,
    sourceRoots: ["cases"],
    sourcePath: "cases/tasks.ts",
    taskSource: "{ id: \"CASE-001\", label: \"case\", run: runCase }",
    runExpression: "runCase"
  });
}

function writeRunCase(root: string, unrelatedValue: string): void {
  writeText(path.join(root, "cases", "run-case.ts"), [
    "import { assertion } from \"./assertion.ts\";",
    "",
    "export async function runCase() {",
    "  assertion();",
    "}",
    "",
    "export function unrelated() {",
    `  return ${JSON.stringify(unrelatedValue)};`,
    "}",
    ""
  ].join("\n"));
}

function writeAssertion(root: string, value: string): void {
  writeText(path.join(root, "cases", "assertion.ts"), [
    "export function assertion() {",
    `  return ${JSON.stringify(value)};`,
    "}",
    ""
  ].join("\n"));
}

function writeText(sourcePath: string, source: string): void {
  fs.mkdirSync(path.dirname(sourcePath), { recursive: true });
  fs.writeFileSync(sourcePath, source, "utf8");
}
