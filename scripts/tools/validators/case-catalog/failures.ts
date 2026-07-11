import { type CaseIdCollection, duplicateIds } from "./case-id-index.ts";
import { CASE_CATALOG_DOC, type DocumentedCaseIndex } from "./documented-cases.ts";
import {
  formatInvalidDocumentedCases,
  formatInvalidCaseCode,
  formatInvalidMarkers,
  formatList,
  formatMarkerPathMismatches,
  formatMissingCaseCode,
  formatMissingCaseProves,
  formatMissingDocs,
  formatMissingMarkers,
  formatPlannedMarkers
} from "./diagnostics.ts";

export function collectCaseCatalogFailures(
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
    formatList(`duplicate case IDs in ${CASE_CATALOG_DOC}`, duplicateDocumentedIds),
    formatList("duplicate source @case markers", duplicateMarkerIds),
    formatInvalidMarkers(invalidMarkers),
    formatInvalidDocumentedCases(documented.invalid),
    formatMissingCaseCode(
      documented.implemented.entries.filter((entry) => entry.codeDeclarations === 0)
    ),
    formatInvalidCaseCode(
      documented.implemented.entries.filter(
        (entry) => entry.codeDeclarations > 0 &&
          (entry.codeDeclarations !== 1 || entry.invalidCode)
      )
    ),
    formatMissingCaseProves(
      [...documented.implemented.entries, ...documented.planned.entries].filter(
        (entry) => entry.provesDeclarations === 0 || !entry.provesContent
      )
    ),
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
