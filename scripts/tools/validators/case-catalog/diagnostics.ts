import { type CaseIdCollection } from "./case-id-index.ts";
import { markerPathMismatches } from "./case-path-mismatches.ts";
import { CASE_CATALOG_DOC, type DocumentedCase } from "./documented-cases.ts";

export function formatInvalidMarkers(markers: CaseIdCollection): string | null {
  if (markers.ids.length === 0) {
    return null;
  }
  return [
    "source @case markers must use CATEGORY-SCOPE-INTENT-NNN:",
    ...markers.ids.map((id) => `  - ${id} found in ${casePaths(markers, id)}`)
  ].join("\n");
}

export function formatMissingMarkers(ids: readonly string[], docs: CaseIdCollection): string | null {
  if (ids.length === 0) {
    return null;
  }
  return [
    "documented case IDs missing @case source markers:",
    ...ids.map((id) => `  - ${id} documented in ${casePaths(docs, id)}`)
  ].join("\n");
}

export function formatInvalidDocumentedCases(entries: readonly DocumentedCase[]): string | null {
  if (entries.length === 0) {
    return null;
  }
  return [
    "documented cases must declare Status: implemented or Status: planned:",
    ...entries.map((entry) => `  - ${entry.id} at ${entry.relPath}:${entry.line}`)
  ].join("\n");
}

export function formatMissingCaseCode(entries: readonly DocumentedCase[]): string | null {
  if (entries.length === 0) {
    return null;
  }
  return [
    "implemented documented cases must declare Code:",
    ...entries.map((entry) => `  - ${entry.id} at ${entry.relPath}:${entry.line}`)
  ].join("\n");
}

export function formatInvalidCaseCode(entries: readonly DocumentedCase[]): string | null {
  if (entries.length === 0) {
    return null;
  }
  return [
    "documented cases must declare exactly one Code path:",
    ...entries.map((entry) => `  - ${entry.id} at ${entry.relPath}:${entry.line}`)
  ].join("\n");
}

export function formatMissingCaseProves(entries: readonly DocumentedCase[]): string | null {
  if (entries.length === 0) {
    return null;
  }
  return [
    "documented cases must include non-empty Proves:",
    ...entries.map((entry) => `  - ${entry.id} at ${entry.relPath}:${entry.line}`)
  ].join("\n");
}

export function formatMissingDocs(ids: readonly string[], markers: CaseIdCollection): string | null {
  if (ids.length === 0) {
    return null;
  }
  return [
    `source @case markers missing from ${CASE_CATALOG_DOC}:`,
    ...ids.map((id) => `  - ${id} found in ${casePaths(markers, id)}`)
  ].join("\n");
}

export function formatPlannedMarkers(ids: readonly string[], markers: CaseIdCollection): string | null {
  if (ids.length === 0) {
    return null;
  }
  return [
    "planned cases must not have source @case markers yet:",
    ...ids.map((id) => `  - ${id} found in ${casePaths(markers, id)}`)
  ].join("\n");
}

export function formatMarkerPathMismatches(
  entries: readonly DocumentedCase[],
  markers: CaseIdCollection
): string | null {
  const mismatches = entries.flatMap((entry) => markerPathMismatches(entry, markers));

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

export function formatList(label: string, values: readonly string[]): string | null {
  if (values.length === 0) {
    return null;
  }
  return [label, ...values.map((value) => `  - ${value}`)].join("\n");
}

function casePaths(cases: CaseIdCollection, id: string): string {
  return (cases.byId.get(id) ?? []).join(", ");
}
