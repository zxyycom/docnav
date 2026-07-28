import fs from "node:fs";

import {
  diagnostic,
  type TestEvidenceDiagnostic
} from "../model.ts";
import {
  isSafeRelativePosixPath,
  resolveExistingWorkspacePath
} from "../relative-path.ts";
import type { SemanticTestCase } from "./model.ts";

export function isOwnerRef(value: string): boolean {
  const separator = value.indexOf("#");
  const sourcePath = value.slice(0, separator);
  const heading = value.slice(separator + 1);
  return (
    separator > 0 &&
    separator === value.lastIndexOf("#") &&
    sourcePath.endsWith(".md") &&
    isSafeRelativePosixPath(sourcePath) &&
    heading.length > 0 &&
    heading.trim() === heading &&
    !/\s/.test(heading)
  );
}

export function diagnoseOwnerRefs(
  cases: readonly SemanticTestCase[],
  workspaceRoot: string,
  diagnostics: TestEvidenceDiagnostic[]
): void {
  const anchorsByPath = new Map<string, Set<string> | Error>();
  for (const testCase of cases) {
    if (testCase.ownerRef.length === 0) {
      continue;
    }
    const [sourcePath, heading] = testCase.ownerRef.split("#");
    let anchors = anchorsByPath.get(sourcePath);
    if (anchors === undefined) {
      anchors = readOwnerAnchors(workspaceRoot, sourcePath);
      anchorsByPath.set(sourcePath, anchors);
    }
    diagnoseOwnerHeading(testCase, heading, anchors, diagnostics);
  }
}

function readOwnerAnchors(
  workspaceRoot: string,
  sourcePath: string
): Set<string> | Error {
  try {
    const resolved = resolveExistingWorkspacePath(
      workspaceRoot,
      sourcePath,
      `Case Owner ${sourcePath}`
    );
    if (!resolved.stats.isFile()) {
      throw new Error(`Case Owner ${sourcePath} must be a regular file`);
    }
    return markdownHeadingAnchors(
      fs.readFileSync(resolved.absolutePath, "utf8")
    );
  } catch (error) {
    return error instanceof Error ? error : new Error(String(error));
  }
}

function diagnoseOwnerHeading(
  testCase: SemanticTestCase,
  heading: string,
  anchors: Set<string> | Error,
  diagnostics: TestEvidenceDiagnostic[]
): void {
  if (anchors instanceof Error) {
    diagnostics.push(caseDiagnostic(
      "case.owner-unknown",
      `Case ${testCase.id} Owner cannot be resolved: ${anchors.message}`,
      testCase
    ));
  } else if (!anchors.has(heading)) {
    diagnostics.push(caseDiagnostic(
      "case.owner-heading-unknown",
      `Case ${testCase.id} Owner heading does not exist: ${testCase.ownerRef}`,
      testCase
    ));
  }
}

function markdownHeadingAnchors(source: string): Set<string> {
  const anchors = new Set<string>();
  const repetitions = new Map<string, number>();
  const lines = source.split(/\r?\n/u);
  let cursor = skipDocumentFrontmatter(lines);
  let fence: { marker: "`" | "~"; length: number } | undefined;
  for (; cursor < lines.length; cursor += 1) {
    const line = cursor === 0
      ? (lines[cursor] ?? "").replace(/^\uFEFF/u, "")
      : (lines[cursor] ?? "");
    if (fence !== undefined) {
      if (closesFence(line, fence)) {
        fence = undefined;
      }
      continue;
    }
    const openingFence = readOpeningFence(line);
    if (openingFence !== undefined) {
      fence = openingFence;
      continue;
    }
    const match = /^#{1,6}[ \t]+(.+?)[ \t]*#*[ \t]*$/u.exec(line);
    if (match === null) {
      continue;
    }
    const base = match[1]
      .toLowerCase()
      .replace(/[^\p{Letter}\p{Mark}\p{Number}\s_-]/gu, "")
      .replace(/\s/gu, "-");
    const occurrence = repetitions.get(base) ?? 0;
    anchors.add(occurrence === 0 ? base : `${base}-${occurrence}`);
    repetitions.set(base, occurrence + 1);
  }
  return anchors;
}

function skipDocumentFrontmatter(lines: readonly string[]): number {
  if ((lines[0] ?? "").replace(/^\uFEFF/u, "").trim() !== "---") {
    return 0;
  }
  for (let index = 1; index < lines.length; index += 1) {
    const line = (lines[index] ?? "").trim();
    if (line === "---" || line === "...") {
      return index + 1;
    }
  }
  return lines.length;
}

function readOpeningFence(
  line: string
): { marker: "`" | "~"; length: number } | undefined {
  const match = /^ {0,3}(`{3,}|~{3,})/u.exec(line);
  if (match === null) {
    return undefined;
  }
  return {
    marker: match[1][0] as "`" | "~",
    length: match[1].length
  };
}

function closesFence(
  line: string,
  fence: { marker: "`" | "~"; length: number }
): boolean {
  const match = /^ {0,3}(`+|~+)[ \t]*$/u.exec(line);
  return (
    match !== null &&
    match[1][0] === fence.marker &&
    match[1].length >= fence.length
  );
}

function caseDiagnostic(
  code: string,
  message: string,
  testCase: SemanticTestCase
): TestEvidenceDiagnostic {
  return diagnostic(code, "case", message, {
    caseId: testCase.id,
    path: testCase.sourcePath,
    line: testCase.sourceLine
  });
}
