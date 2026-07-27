import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { releaseComponents } from "../tools/release-package/config.ts";
import {
  AST_GREP_PACKAGE,
  AST_GREP_VERSION,
  expectedAstGrepVersionLine,
  resolveAstGrepExecutable,
  runAstGrep
} from "./ast-grep.ts";

const workspaceRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  ".."
);

test("uses the repository-locked ast-grep CLI through the project wrapper", async () => {
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(workspaceRoot, "package.json"), "utf8")
  ) as { devDependencies?: Record<string, string> };
  assert.equal(packageJson.devDependencies?.[AST_GREP_PACKAGE], AST_GREP_VERSION);
  assert.equal(
    resolveAstGrepExecutable(),
    path.join(
      workspaceRoot,
      "node_modules",
      AST_GREP_PACKAGE,
      process.platform === "win32" ? "ast-grep.exe" : "ast-grep"
    )
  );

  const result = await runAstGrep(["--version"]);
  assert.equal(result.status, 0);
  assert.equal(result.stdout.trim(), expectedAstGrepVersionLine());
});

test("pins the audited ast-grep package resolutions in the repository lockfile", () => {
  const lockfile = fs.readFileSync(
    path.join(workspaceRoot, "pnpm-lock.yaml"),
    "utf8"
  );
  assert.match(
    lockfile,
    /'@ast-grep\/cli':\n\s+specifier: 0\.45\.0\n\s+version: 0\.45\.0/
  );
  assert.ok(lockfile.includes(
    "'@ast-grep/cli@0.45.0':\n" +
    "    resolution: {integrity: sha512-OQ4pcktMtg1hcQat/iCpX9r8HJ7mU/2SZVoGHA9id2gEfosvDw5m5RINQXsSRZXQW8bl45FW6FhdK0O2FiKjsw==}"
  ));
  assert.ok(lockfile.includes(
    "'@ast-grep/cli-linux-x64-gnu@0.45.0':\n" +
    "    resolution: {integrity: sha512-rAMZJzAiBuXMViuJgdPeMZXI9HnqwMCh3ybIoj8dfWBPsAywKgU8vyH4kd/R5fFr/oB4lKVhTJ2/mEBsOQTHaQ==}"
  ));
});

test("keeps the pinned ast-grep skill distribution byte-for-byte complete", () => {
  const skillRoot = path.join(workspaceRoot, ".codex", "skills", "ast-grep");
  assert.deepEqual(listRelativeFiles(skillRoot), [
    "SKILL.md",
    "agents/openai.yaml",
    "references/rules-and-recipes.md",
    "scripts/update-skill.d.mts",
    "scripts/update-skill.mjs",
    "scripts/update-skill.mjs.map"
  ]);
  assert.equal(
    directoryFingerprint(skillRoot),
    "8957af003ca667e987db9e42e7f76e8f6813a0fe9f7e87a09ce4454424de0d44"
  );
});

test("keeps the project-owned test-evidence skill complete and updater-free", () => {
  const skillRoot = path.join(
    workspaceRoot,
    ".codex",
    "skills",
    "test-evidence-review"
  );
  assert.deepEqual(listRelativeFiles(skillRoot), [
    "SKILL.md",
    "agents/openai.yaml",
    "references/evidence-contract.md",
    "schemas/claim-topic-catalog.schema.json",
    "schemas/evidence-claim.schema.json",
    "schemas/native-test-entry.schema.json",
    "schemas/native-test-inventory.schema.json",
    "schemas/test-evidence-index.schema.json",
    "scripts/test-evidence-catalog.d.mts",
    "scripts/test-evidence-catalog.mjs"
  ]);
  assert.equal(
    directoryFingerprint(skillRoot),
    "38fa7fe98879b5f1bae042734fc4c92817228ba7c0479d22e3a12ab2846dc7f8"
  );
});

test("keeps the development ast-grep executable outside canonical release components", () => {
  assert.deepEqual(releaseComponents, [
    {
      component: "core",
      packageName: "docnav",
      binName: "docnav"
    }
  ]);
  assert.ok(
    releaseComponents.every(({ binName, packageName }) => (
      !binName.includes("ast-grep") && !packageName.includes("ast-grep")
    ))
  );
});

test("does not invoke the external ast-grep executable outside the developer wrapper", () => {
  const directInvocationPatterns = [
    /\bcommand\s*:\s*["'](?:ast-grep|sg)["']/,
    /\bCommand::new\(\s*["'](?:ast-grep|sg)["']\s*\)/,
    /\b(?:spawn|spawnSync|exec|execFile|execFileSync)\(\s*["'](?:ast-grep|sg)["']/
  ];
  const sourcePaths = [
    ...listFilesWithExtension(path.join(workspaceRoot, "crates"), ".rs"),
    ...listFilesWithExtension(path.join(workspaceRoot, "scripts"), ".ts"),
    ...listFilesWithExtension(path.join(workspaceRoot, "test"), ".ts")
  ].filter((sourcePath) => sourcePath !== path.join(
    workspaceRoot,
    "scripts",
    "test-evidence",
    "ast-grep.ts"
  ));

  const violations = sourcePaths.flatMap((sourcePath) => {
    const source = fs.readFileSync(sourcePath, "utf8");
    return directInvocationPatterns
      .filter((pattern) => pattern.test(source))
      .map((pattern) => `${path.relative(workspaceRoot, sourcePath)}: ${String(pattern)}`);
  });
  assert.deepEqual(violations, []);
});

function directoryFingerprint(root: string): string {
  const manifest = listRelativeFiles(root)
    .map((relativePath) => {
      const contentHash = crypto
        .createHash("sha256")
        .update(fs.readFileSync(path.join(root, relativePath)))
        .digest("hex");
      return `${relativePath}\0${contentHash}\n`;
    })
    .join("");
  return crypto.createHash("sha256").update(manifest).digest("hex");
}

function listFilesWithExtension(root: string, extension: string): string[] {
  if (!fs.existsSync(root)) {
    return [];
  }
  const files: string[] = [];
  walk(root);
  return files.sort();

  function walk(directory: string): void {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const entryPath = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        walk(entryPath);
      } else if (entry.isFile() && entryPath.endsWith(extension)) {
        files.push(entryPath);
      }
    }
  }
}

function listRelativeFiles(root: string): string[] {
  const files: string[] = [];
  walk(root, "");
  return files.sort();

  function walk(directory: string, relativeDirectory: string): void {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const relativePath = path.posix.join(relativeDirectory, entry.name);
      const absolutePath = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        walk(absolutePath, relativePath);
      } else if (entry.isFile()) {
        files.push(relativePath);
      }
    }
  }
}
