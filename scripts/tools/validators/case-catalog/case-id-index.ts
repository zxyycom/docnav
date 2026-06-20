export interface CaseIdCollection {
  byId: Map<string, string[]>;
  allIds: string[];
  ids: string[];
}

export function collectionFromEntries(
  entries: readonly (readonly [string, string])[]
): CaseIdCollection {
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

export function duplicateIds(ids: readonly string[]): string[] {
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

