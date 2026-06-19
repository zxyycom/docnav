import { assert } from "./fs-utils.ts";
import { readText } from "./document-files.ts";
import { collectionFromEntries } from "./case-catalog/collection.ts";
import {
  CASE_CATALOG_DOC,
  collectDocumentedCasesFromText,
  type DocumentedCaseIndex
} from "./case-catalog/documented.ts";
import { collectCaseCatalogFailures } from "./case-catalog/failures.ts";
import {
  CASE_ID_EXACT_PATTERN,
  caseSourceFiles,
  collectCaseMarkers,
  collectInvalidCaseMarkers
} from "./case-catalog/markers.ts";

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

function collectDocumentedCases(): DocumentedCaseIndex {
  return collectDocumentedCasesFromText(readText(CASE_CATALOG_DOC));
}
