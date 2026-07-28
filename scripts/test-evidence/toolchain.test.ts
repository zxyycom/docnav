import assert from "node:assert/strict";
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
