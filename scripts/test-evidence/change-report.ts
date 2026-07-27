import type {
  NativeTestEntry,
  NativeTestInventory
} from "./model.ts";

export type NativeTestChangeReport = {
  schemaVersion: 1;
  baselineRevision: string;
  currentRevision: string;
  added: string[];
  removed: string[];
  implementationChanged: string[];
  renameCandidates: Array<{
    from: string;
    to: string;
  }>;
};

export function compareInventoryBaseline(
  baseline: NativeTestInventory,
  current: NativeTestInventory
): NativeTestChangeReport {
  const baselineByKey = new Map(
    baseline.entries.map((entry) => [entry.entryKey, entry])
  );
  const currentByKey = new Map(
    current.entries.map((entry) => [entry.entryKey, entry])
  );
  const added = current.entries
    .filter(({ entryKey }) => !baselineByKey.has(entryKey));
  const removed = baseline.entries
    .filter(({ entryKey }) => !currentByKey.has(entryKey));
  const implementationChanged = current.entries
    .filter((entry) => {
      const oldEntry = baselineByKey.get(entry.entryKey);
      return (
        oldEntry !== undefined &&
        oldEntry.sourceFingerprint !== entry.sourceFingerprint
      );
    })
    .map(({ entryKey }) => entryKey)
    .sort();
  const renameCandidates = createRenameCandidates(removed, added);

  return {
    schemaVersion: 1,
    baselineRevision: baseline.sourceRevision,
    currentRevision: current.sourceRevision,
    added: added.map(({ entryKey }) => entryKey).sort(),
    removed: removed.map(({ entryKey }) => entryKey).sort(),
    implementationChanged,
    renameCandidates
  };
}

function createRenameCandidates(
  removed: readonly NativeTestEntry[],
  added: readonly NativeTestEntry[]
): Array<{ from: string; to: string }> {
  const candidates: Array<{ from: string; to: string }> = [];
  for (const oldEntry of removed) {
    const possible = added.filter((newEntry) => (
      newEntry.runner === oldEntry.runner &&
      newEntry.target === oldEntry.target &&
      newEntry.sourcePath === oldEntry.sourcePath &&
      Math.abs(
        newEntry.sourceRange.startLine - oldEntry.sourceRange.startLine
      ) <= 3
    ));
    if (possible.length === 1) {
      candidates.push({
        from: oldEntry.entryKey,
        to: possible[0].entryKey
      });
    }
  }
  return candidates.sort((left, right) => (
    left.from < right.from ? -1 : left.from > right.from ? 1 : 0
  ));
}
