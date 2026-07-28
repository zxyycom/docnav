import {
  diagnostic,
  type RuntimeTestEntity,
  type StaticTestEntity,
  type TestEntity,
  type TestEvidenceDiagnostic
} from "./model.ts";

export function closeStaticAndRuntimeEntities(options: {
  runner: string;
  statics: StaticTestEntity[];
  runtime: RuntimeTestEntity[];
  createEntityKey: (runtime: RuntimeTestEntity) => string;
}): {
  entities: TestEntity[];
  diagnostics: TestEvidenceDiagnostic[];
} {
  const diagnostics: TestEvidenceDiagnostic[] = [];
  const staticGroups = groupByIdentity(options.statics);
  const runtimeGroups = groupByIdentity(options.runtime);
  const identities = [...new Set([
    ...staticGroups.keys(),
    ...runtimeGroups.keys()
  ])].sort();
  const entities: TestEntity[] = [];

  for (const identity of identities) {
    const staticCandidates = staticGroups.get(identity) ?? [];
    const runtimeEntries = runtimeGroups.get(identity) ?? [];
    if (staticCandidates.length > 1 || runtimeEntries.length > 1) {
      diagnostics.push(diagnostic(
        "duplicate-entity",
        staticCandidates.length > 1 ? "static" : "runner",
        `${options.runner} TestEntity identity ${identity} is ambiguous (${staticCandidates.length} static, ${runtimeEntries.length} runtime)`,
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
        `${options.runner} static TestEntity ${identity} is absent from the runner report`,
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
        `${options.runner} runtime TestEntity ${runtime.selector} has no supported static declaration`,
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
    entities.push({
      entityKey: options.createEntityKey(runtime),
      runner: options.runner,
      target: runtime.target,
      selector: runtime.selector,
      sourcePath: candidate.sourcePath,
      sourceRange: candidate.sourceRange
    });
  }

  return {
    entities: entities.sort((left, right) => (
      left.entityKey < right.entityKey ? -1 : left.entityKey > right.entityKey ? 1 : 0
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
