import { type CaseIdCollection } from "./case-id-index.ts";
import { normalizeCasePath } from "./case-paths.ts";
import { type DocumentedCase } from "./documented-cases.ts";

interface MarkerPathMismatch {
  id: string;
  expected: string;
  actual: string;
}

export function markerPathMismatches(
  entry: DocumentedCase,
  markers: CaseIdCollection
): MarkerPathMismatch[] {
  const expected = entry.codePath;
  if (expected === null) {
    return [];
  }
  return (markers.byId.get(entry.id) ?? [])
    .filter((actual) => normalizeCasePath(actual) !== expected)
    .map((actual) => ({ id: entry.id, expected, actual }));
}
