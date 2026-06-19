import fs from "node:fs";
import path from "node:path";

import { FILE_SYSTEM } from "./config.ts";
import { assert, toAbs, toRel, walk } from "./fs-utils.ts";
import { readText } from "./document-files.ts";

const CASE_CATALOG_DOC = "docs/testing/cases.md";
const SOURCE_ROOTS = ["test", "crates", "scripts"];
const SOURCE_EXTENSIONS = new Set([".rs", ".ts"]);
const CASE_ID_SOURCE = "(?:BB|WB|AUX)(?:-[A-Z0-9]+){2,}-\\d{3}";
const CASE_ID_EXACT_PATTERN = new RegExp(`^${CASE_ID_SOURCE}$`, "u");
const CASE_HEADING_PATTERN = new RegExp(`^###+\\s+(${CASE_ID_SOURCE})\\b`, "gmu");
const CASE_MARKER_PATTERN = /^\s*(?:(?:\/\/)|#)\s*@case\s+(\S+)/gmu;

interface CaseIdCollection {
  byId: Map<string, string[]>;
  allIds: string[];
  ids: string[];
}

export function validateTestCaseCatalog(): void {
  const documented = collectDocumentedCaseIds();
  const markers = collectCaseMarkers(caseSourceFiles());
  const invalidMarkers = collectInvalidCaseMarkers(caseSourceFiles());
  const duplicateDocumentedIds = duplicateIds(documented.allIds);
  const documentedSet = new Set(documented.ids);
  const markerSet = new Set(markers.ids);

  const failures = [
    formatList("duplicate case IDs in docs/testing/cases.md", duplicateDocumentedIds),
    formatInvalidMarkers(invalidMarkers),
    formatMissingMarkers(documented.ids.filter((id) => !markerSet.has(id)), documented),
    formatMissingDocs(markers.ids.filter((id) => !documentedSet.has(id)), markers)
  ].filter((message): message is string => message !== null);

  assert(failures.length === 0, `test case catalog drift:\n${failures.join("\n\n")}`);
  console.log(`test case catalog ok: ${documented.ids.length} documented, ${markers.ids.length} source marker(s)`);
}

function collectDocumentedCaseIds(): CaseIdCollection {
  const ids = extractDocumentedCaseIds(readText(CASE_CATALOG_DOC));
  return collectionFromEntries(ids.map((id) => [id, CASE_CATALOG_DOC]));
}

function extractDocumentedCaseIds(text: string): string[] {
  return [...text.matchAll(CASE_HEADING_PATTERN)].map((match) => match[1] ?? "");
}

function collectCaseMarkers(relPaths: readonly string[]): CaseIdCollection {
  return collectionFromEntries(
    relPaths.flatMap((relPath) =>
      extractCaseMarkerIds(readText(relPath))
        .filter((id) => CASE_ID_EXACT_PATTERN.test(id))
        .map((id) => [id, relPath] as const)
    )
  );
}

function collectInvalidCaseMarkers(relPaths: readonly string[]): CaseIdCollection {
  return collectionFromEntries(
    relPaths.flatMap((relPath) =>
      extractCaseMarkerIds(readText(relPath))
        .filter((id) => !CASE_ID_EXACT_PATTERN.test(id))
        .map((id) => [id, relPath] as const)
    )
  );
}

function extractCaseMarkerIds(text: string): string[] {
  return [...text.matchAll(CASE_MARKER_PATTERN)].map((match) => match[1] ?? "");
}

function collectionFromEntries(entries: readonly (readonly [string, string])[]): CaseIdCollection {
  const byId = new Map<string, string[]>();
  for (const [id, relPath] of entries) {
    const paths = byId.get(id) ?? [];
    paths.push(relPath);
    byId.set(id, paths);
  }

  return {
    byId,
    allIds: entries.map(([id]) => id).sort(),
    ids: [...byId.keys()].sort()
  };
}

function duplicateIds(ids: readonly string[]): string[] {
  const seen = new Set<string>();
  const duplicates = new Set<string>();
  for (const id of ids) {
    if (seen.has(id)) {
      duplicates.add(id);
    }
    seen.add(id);
  }
  return [...duplicates].sort();
}

function caseSourceFiles(): string[] {
  return SOURCE_ROOTS.flatMap((root) => {
    const absRoot = toAbs(root);
    if (!fs.existsSync(absRoot)) {
      return [];
    }
    return walk(absRoot, isCaseSourceFile).map(toRel);
  }).sort();
}

function isCaseSourceFile(filePath: string): boolean {
  const relPath = toRel(filePath);
  if (FILE_SYSTEM.ignoredDirs.some((dir) => relPath.split("/").includes(dir))) {
    return false;
  }
  return SOURCE_EXTENSIONS.has(path.extname(filePath));
}

function formatInvalidMarkers(markers: CaseIdCollection): string | null {
  if (markers.ids.length === 0) {
    return null;
  }
  return [
    "source @case markers must use CATEGORY-SCOPE-INTENT-NNN:",
    ...markers.ids.map((id) => `  - ${id} found in ${casePaths(markers, id)}`)
  ].join("\n");
}

function formatMissingMarkers(ids: readonly string[], docs: CaseIdCollection): string | null {
  if (ids.length === 0) {
    return null;
  }
  return [
    "documented case IDs missing @case source markers:",
    ...ids.map((id) => `  - ${id} documented in ${casePaths(docs, id)}`)
  ].join("\n");
}

function formatMissingDocs(ids: readonly string[], markers: CaseIdCollection): string | null {
  if (ids.length === 0) {
    return null;
  }
  return [
    "source @case markers missing from docs/testing/cases.md:",
    ...ids.map((id) => `  - ${id} found in ${casePaths(markers, id)}`)
  ].join("\n");
}

function formatList(label: string, values: readonly string[]): string | null {
  if (values.length === 0) {
    return null;
  }
  return [label, ...values.map((value) => `  - ${value}`)].join("\n");
}

function casePaths(cases: CaseIdCollection, id: string): string {
  return (cases.byId.get(id) ?? []).join(", ");
}
