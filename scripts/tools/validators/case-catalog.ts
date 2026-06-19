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
const CASE_MARKER_PATTERN = /^\s*(?:(?:\/\/)|#)\s*@case\s+(\S+)/gmu;
const CASE_STATUS_PATTERN = /^Status:\s+(\S+)/u;
const CASE_CODE_PATTERN = /^Code:\s+`([^`]+)`/u;

interface CaseIdCollection {
  byId: Map<string, string[]>;
  allIds: string[];
  ids: string[];
}

interface DocumentedCase {
  id: string;
  relPath: string;
  status: CaseStatus | null;
  line: number;
  codePath: string | null;
}

interface DocumentedCaseIndex {
  allIds: string[];
  ids: string[];
  implemented: CaseIdCollection & { entries: DocumentedCase[] };
  planned: CaseIdCollection & { entries: DocumentedCase[] };
  invalid: DocumentedCase[];
}

type CaseStatus = "implemented" | "planned";

export interface CaseCatalogMarker {
  id: string;
  relPath: string;
}

export interface CaseCatalogSnapshot {
  catalogText: string;
  markers: readonly CaseCatalogMarker[];
}

export function validateTestCaseCatalog(): void {
  const documented = collectDocumentedCases();
  const markers = collectCaseMarkers(caseSourceFiles());
  const invalidMarkers = collectInvalidCaseMarkers(caseSourceFiles());
  const failures = collectCaseCatalogFailures(documented, markers, invalidMarkers);

  assert(failures.length === 0, `test case catalog drift:\n${failures.join("\n\n")}`);

  const planned = documented.planned.ids.length;
  const implemented = documented.implemented.ids.length;
  console.log(
    `test case catalog ok: ${implemented} implemented, ${planned} planned, ${markers.ids.length} source marker(s)`
  );
}

export function validateCaseCatalogSnapshot(snapshot: CaseCatalogSnapshot): string[] {
  const documented = collectDocumentedCasesFromText(snapshot.catalogText);
  const markers = collectionFromEntries(
    snapshot.markers
      .filter((marker) => CASE_ID_EXACT_PATTERN.test(marker.id))
      .map((marker) => [marker.id, marker.relPath] as const)
  );
  const invalidMarkers = collectionFromEntries(
    snapshot.markers
      .filter((marker) => !CASE_ID_EXACT_PATTERN.test(marker.id))
      .map((marker) => [marker.id, marker.relPath] as const)
  );

  return collectCaseCatalogFailures(documented, markers, invalidMarkers);
}

function collectCaseCatalogFailures(
  documented: DocumentedCaseIndex,
  markers: CaseIdCollection,
  invalidMarkers: CaseIdCollection
): string[] {
  const duplicateDocumentedIds = duplicateIds(documented.allIds);
  const duplicateMarkerIds = duplicateIds(markers.allIds);
  const documentedSet = new Set(documented.ids);
  const plannedSet = new Set(documented.planned.ids);
  const markerSet = new Set(markers.ids);

  return [
    formatList("duplicate case IDs in docs/testing/cases.md", duplicateDocumentedIds),
    formatList("duplicate source @case markers", duplicateMarkerIds),
    formatInvalidMarkers(invalidMarkers),
    formatInvalidDocumentedCases(documented.invalid),
    formatMissingCaseCode(documented.implemented.entries.filter((entry) => entry.codePath === null)),
    formatMissingMarkers(
      documented.implemented.ids.filter((id) => !markerSet.has(id)),
      documented.implemented
    ),
    formatMissingDocs(markers.ids.filter((id) => !documentedSet.has(id)), markers),
    formatPlannedMarkers(markers.ids.filter((id) => plannedSet.has(id)), markers),
    formatMarkerPathMismatches(
      documented.implemented.entries.filter(
        (entry) => entry.codePath !== null && markerSet.has(entry.id)
      ),
      markers
    )
  ].filter((message): message is string => message !== null);
}

function collectDocumentedCases(): DocumentedCaseIndex {
  return collectDocumentedCasesFromText(readText(CASE_CATALOG_DOC));
}

function collectDocumentedCasesFromText(text: string): DocumentedCaseIndex {
  const entries = extractDocumentedCases(text);
  const implemented = entries.filter((entry) => entry.status === "implemented");
  const planned = entries.filter((entry) => entry.status === "planned");
  return {
    allIds: entries.map((entry) => entry.id).sort(),
    ids: [...new Set(entries.map((entry) => entry.id))].sort(),
    implemented: {
      ...collectionFromEntries(implemented.map((entry) => [entry.id, CASE_CATALOG_DOC] as const)),
      entries: implemented
    },
    planned: {
      ...collectionFromEntries(planned.map((entry) => [entry.id, CASE_CATALOG_DOC] as const)),
      entries: planned
    },
    invalid: entries.filter((entry) => entry.status === null)
  };
}

function extractDocumentedCases(text: string): DocumentedCase[] {
  const lines = text.split(/\r?\n/u);
  const entries: DocumentedCase[] = [];
  let current: DocumentedCase | null = null;

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index] ?? "";
    const heading = line.match(/^###+\s+((?:BB|WB|AUX)(?:-[A-Z0-9]+){2,}-\d{3})\b/u);
    if (heading !== null) {
      current = {
        id: heading[1] ?? "",
        relPath: CASE_CATALOG_DOC,
        status: null,
        line: index + 1,
        codePath: null
      };
      entries.push(current);
      continue;
    }

    if (current === null) {
      continue;
    }

    const status = line.match(CASE_STATUS_PATTERN);
    if (status !== null) {
      current.status = parseCaseStatus(status[1] ?? "");
      continue;
    }

    const code = line.match(CASE_CODE_PATTERN);
    if (code !== null) {
      current.codePath = normalizeCasePath(code[1] ?? "");
    }
  }

  return entries;
}

function parseCaseStatus(value: string): CaseStatus | null {
  if (value === "implemented" || value === "planned") {
    return value;
  }
  return null;
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

function formatInvalidDocumentedCases(entries: readonly DocumentedCase[]): string | null {
  if (entries.length === 0) {
    return null;
  }
  return [
    "documented cases must declare Status: implemented or Status: planned:",
    ...entries.map((entry) => `  - ${entry.id} at ${entry.relPath}:${entry.line}`)
  ].join("\n");
}

function formatMissingCaseCode(entries: readonly DocumentedCase[]): string | null {
  if (entries.length === 0) {
    return null;
  }
  return [
    "implemented documented cases must declare Code:",
    ...entries.map((entry) => `  - ${entry.id} at ${entry.relPath}:${entry.line}`)
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

function formatPlannedMarkers(ids: readonly string[], markers: CaseIdCollection): string | null {
  if (ids.length === 0) {
    return null;
  }
  return [
    "planned cases must not have source @case markers yet:",
    ...ids.map((id) => `  - ${id} found in ${casePaths(markers, id)}`)
  ].join("\n");
}

function formatMarkerPathMismatches(
  entries: readonly DocumentedCase[],
  markers: CaseIdCollection
): string | null {
  const mismatches = entries.flatMap((entry) => {
    const paths = markers.byId.get(entry.id) ?? [];
    const expected = entry.codePath;
    if (expected === null) {
      return [];
    }
    return paths
      .filter((actual) => normalizeCasePath(actual) !== expected)
      .map((actual) => ({ id: entry.id, expected, actual }));
  });

  if (mismatches.length === 0) {
    return null;
  }

  return [
    "documented Code paths must match source @case marker paths:",
    ...mismatches.map(
      (mismatch) => `  - ${mismatch.id} documented ${mismatch.expected}, marker in ${mismatch.actual}`
    )
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

function normalizeCasePath(value: string): string {
  return value.replaceAll("\\", "/");
}
