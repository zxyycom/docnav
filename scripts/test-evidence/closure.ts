import {
  diagnostic,
  type NativeTestEntry,
  type RuntimeTestEntry,
  type StaticTestCandidate,
  type TestEvidenceDiagnostic
} from "./model.ts";

export function closeStaticAndRuntimeEntries(options: {
  runner: string;
  statics: StaticTestCandidate[];
  runtime: RuntimeTestEntry[];
  createEntryKey: (runtime: RuntimeTestEntry) => string;
}): {
  entries: NativeTestEntry[];
  diagnostics: TestEvidenceDiagnostic[];
} {
  const diagnostics: TestEvidenceDiagnostic[] = [];
  const staticGroups = groupByIdentity(options.statics);
  const runtimeGroups = groupByIdentity(options.runtime);
  const identities = [...new Set([
    ...staticGroups.keys(),
    ...runtimeGroups.keys()
  ])].sort();
  const entries: NativeTestEntry[] = [];

  for (const identity of identities) {
    const staticCandidates = staticGroups.get(identity) ?? [];
    const runtimeEntries = runtimeGroups.get(identity) ?? [];
    if (staticCandidates.length > 1 || runtimeEntries.length > 1) {
      diagnostics.push(diagnostic(
        "duplicate-entry",
        staticCandidates.length > 1 ? "static" : "runner",
        `${options.runner} identity ${identity} is ambiguous (${staticCandidates.length} static, ${runtimeEntries.length} runtime)`,
        {
          runner: options.runner,
          selector: runtimeEntries[0]?.selector,
          path: staticCandidates[0]?.sourcePath
        }
      ));
      continue;
    }
    if (staticCandidates.length === 1 && runtimeEntries.length === 0) {
      const candidate = staticCandidates[0];
      diagnostics.push(diagnostic(
        "static-only",
        "static",
        `${options.runner} static entry ${identity} is absent from the runner report`,
        {
          runner: options.runner,
          path: candidate.sourcePath,
          line: candidate.sourceRange.startLine,
          column: candidate.sourceRange.startColumn
        }
      ));
      continue;
    }
    if (staticCandidates.length === 0 && runtimeEntries.length === 1) {
      const runtime = runtimeEntries[0];
      diagnostics.push(diagnostic(
        "runtime-only",
        "runner",
        `${options.runner} runner entry ${runtime.selector} has no supported static declaration`,
        {
          runner: options.runner,
          target: runtime.target,
          selector: runtime.selector
        }
      ));
      continue;
    }
    const candidate = staticCandidates[0];
    const runtime = runtimeEntries[0];
    if (!candidate || !runtime) {
      continue;
    }
    entries.push({
      entryKey: options.createEntryKey(runtime),
      runner: options.runner,
      target: runtime.target,
      selector: runtime.selector,
      sourcePath: candidate.sourcePath,
      sourceRange: candidate.sourceRange,
      sourceFingerprint: candidate.sourceFingerprint
    });
  }

  return {
    entries: entries.sort((left, right) => (
      left.entryKey < right.entryKey ? -1 : left.entryKey > right.entryKey ? 1 : 0
    )),
    diagnostics
  };
}

function groupByIdentity<T extends { identity: string }>(
  values: readonly T[]
): Map<string, T[]> {
  const groups = new Map<string, T[]>();
  for (const value of values) {
    const group = groups.get(value.identity) ?? [];
    group.push(value);
    groups.set(value.identity, group);
  }
  return groups;
}
