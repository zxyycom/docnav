import fs from "node:fs";
import path from "node:path";

import { FILE_SYSTEM } from "../config.ts";
import { readText } from "../document/files.ts";
import { toAbs, toRel, walk } from "../fs/utils.ts";
import { collectionFromEntries, type CaseIdCollection } from "./collection.ts";

const SOURCE_ROOTS = ["test", "crates", "scripts"];
const SOURCE_EXTENSIONS = new Set([".rs", ".ts"]);
const CASE_ID_SOURCE = "(?:BB|WB|AUX)(?:-[A-Z0-9]+){2,}-\\d{3}";
const CASE_MARKER_PATTERN = /^\s*(?:(?:\/\/)|#)\s*@case\s+(\S+)/gmu;

export const CASE_ID_EXACT_PATTERN = new RegExp(`^${CASE_ID_SOURCE}$`, "u");

export function collectCaseMarkers(relPaths: readonly string[]): CaseIdCollection {
  return collectMarkersByValidity(relPaths, true);
}

export function collectInvalidCaseMarkers(relPaths: readonly string[]): CaseIdCollection {
  return collectMarkersByValidity(relPaths, false);
}

export function caseSourceFiles(): string[] {
  return SOURCE_ROOTS.flatMap((root) => {
    const absRoot = toAbs(root);
    if (!fs.existsSync(absRoot)) {
      return [];
    }
    return walk(absRoot, isCaseSourceFile).map(toRel);
  }).sort();
}

function collectMarkersByValidity(
  relPaths: readonly string[],
  requireValidId: boolean
): CaseIdCollection {
  return collectionFromEntries(
    relPaths.flatMap((relPath) =>
      extractCaseMarkerIds(readText(relPath))
        .filter((id) => CASE_ID_EXACT_PATTERN.test(id) === requireValidId)
        .map((id) => [id, relPath] as const)
    )
  );
}

function extractCaseMarkerIds(text: string): string[] {
  return [...text.matchAll(CASE_MARKER_PATTERN)].map((match) => match[1] ?? "");
}

function isCaseSourceFile(filePath: string): boolean {
  const relPath = toRel(filePath);
  if (FILE_SYSTEM.ignoredDirs.some((dir) => relPath.split("/").includes(dir))) {
    return false;
  }
  return SOURCE_EXTENSIONS.has(path.extname(filePath));
}
