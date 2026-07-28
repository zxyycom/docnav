import path from "node:path";

import {
  prepareSmokeTasks,
  selectSmokeTasks
} from "../../../test/tools/smoke-harness.ts";
import { createCoreSmokeTasks } from "../../../test/smoke/core/profile.ts";
import {
  astSourceRange,
  scanAstRule,
  unsupportedAstDiagnostics,
  type AstMatch
} from "../ast-scan.ts";
import { closeStaticAndRuntimeEntries } from "../closure.ts";
import {
  diagnostic,
  type NativeTestEntry,
  type RuntimeTestEntry,
  type StaticTestCandidate,
  type TestEvidenceDiagnostic
} from "../model.ts";
import {
  SUPPORTED_SMOKE_FACTORY,
  type SupportedRunnerProfile
} from "../profile.ts";
import { createSmokeSourceFingerprint } from "./smoke-fingerprint.ts";

type SmokeDiscoveryOptions = {
  workspaceRoot: string;
  profile: SupportedRunnerProfile;
};

export async function discoverSmokeEntries(
  options: SmokeDiscoveryOptions
): Promise<{
  entries: NativeTestEntry[];
  diagnostics: TestEvidenceDiagnostic[];
}> {
  const diagnostics: TestEvidenceDiagnostic[] = [];
  if (options.profile.smoke.factory !== SUPPORTED_SMOKE_FACTORY) {
    return {
      entries: [],
      diagnostics: [
        diagnostic(
          "runner-profile-invalid",
          "profile",
          `unsupported smoke factory ${options.profile.smoke.factory}`,
          {
            path: options.profile.smoke.factory,
            runner: "smoke"
          }
        )
      ]
    };
  }

  const matches = await scanSmokeSources(options, diagnostics);
  const statics = createStaticCandidates(options, matches, diagnostics);
  const runtime = createRuntimeEntries(options.profile, diagnostics);

  if (diagnostics.some(({ blocking }) => blocking)) {
    return {
      entries: [],
      diagnostics
    };
  }
  const closed = closeStaticAndRuntimeEntries({
    runner: "smoke",
    statics,
    runtime,
    createEntryKey: ({ target, selector }) => `smoke|${target}|${selector}`
  });
  return {
    entries: closed.entries,
    diagnostics: [...diagnostics, ...closed.diagnostics]
  };
}

async function scanSmokeSources(
  options: SmokeDiscoveryOptions,
  diagnostics: TestEvidenceDiagnostic[]
): Promise<AstMatch[]> {
  const ruleRoot = path.join(
    options.workspaceRoot,
    "scripts",
    "test-evidence",
    "rules"
  );
  const nativeScan = await scanAstRule({
    workspaceRoot: options.workspaceRoot,
    rulePath: path.join(ruleRoot, "smoke-native-leaf.yml"),
    paths: options.profile.smoke.sourceRoots
  });
  const unsupportedScan = await scanAstRule({
    workspaceRoot: options.workspaceRoot,
    rulePath: path.join(ruleRoot, "smoke-unsupported-dynamic.yml"),
    paths: options.profile.smoke.sourceRoots
  });
  diagnostics.push(
    ...nativeScan.diagnostics,
    ...unsupportedScan.diagnostics,
    ...unsupportedAstDiagnostics(unsupportedScan.matches, "smoke")
  );
  return nativeScan.matches;
}

function createStaticCandidates(
  options: SmokeDiscoveryOptions,
  matches: AstMatch[],
  diagnostics: TestEvidenceDiagnostic[]
): StaticTestCandidate[] {
  const statics: StaticTestCandidate[] = [];
  for (const match of matches) {
    const candidate = createStaticCandidate(options, match, diagnostics);
    if (candidate) {
      statics.push(candidate);
    }
  }
  return statics;
}

function createStaticCandidate(
  options: SmokeDiscoveryOptions,
  match: AstMatch,
  diagnostics: TestEvidenceDiagnostic[]
): StaticTestCandidate | null {
  const id = match.metaVariables.single.ID?.text;
  const runExpression = match.metaVariables.single.RUN?.text;
  if (!id || !runExpression) {
    diagnostics.push(diagnostic(
      "static-scan-failed",
      "static",
      "smoke native leaf rule did not capture ID and RUN",
      {
        path: match.file,
        line: match.range.start.line + 1,
        runner: "smoke"
      }
    ));
    return null;
  }
  try {
    return {
      identity: id,
      sourcePath: match.file,
      sourceRange: astSourceRange(match),
      sourceFingerprint: createSmokeSourceFingerprint({
        workspaceRoot: options.workspaceRoot,
        sourceRoots: options.profile.smoke.sourceRoots,
        sourcePath: match.file,
        taskSource: match.text,
        runExpression
      })
    };
  } catch (error) {
    diagnostics.push(diagnostic(
      "unsupported-entry-shape",
      "static",
      `smoke task ${id} has no attributable implementation: ${
        error instanceof Error ? error.message : String(error)
      }`,
      {
        path: match.file,
        line: match.range.start.line + 1,
        runner: "smoke",
        selector: id
      }
    ));
    return null;
  }
}

function createRuntimeEntries(
  profile: SupportedRunnerProfile,
  diagnostics: TestEvidenceDiagnostic[]
): RuntimeTestEntry[] {
  try {
    const prepared = prepareSmokeTasks(createCoreSmokeTasks());
    return prepared.map((task) => {
      const reportId = typeof task.reportId === "string"
        ? task.reportId
        : null;
      if (!reportId) {
        throw new Error(`smoke leaf ${task.id} has no report root`);
      }
      const selected = selectSmokeTasks(prepared, task.id);
      if (selected.length !== 1 || selected[0]?.id !== task.id) {
        throw new Error(`smoke leaf ${task.id} is not exactly selectable`);
      }
      return {
        identity: task.id,
        target: `${profile.smoke.id}:${reportId}`,
        selector: task.id
      };
    });
  } catch (error) {
    diagnostics.push(diagnostic(
      "runner-list-failed",
      "runner",
      `smoke task expansion failed: ${error instanceof Error ? error.message : String(error)}`,
      {
        path: profile.smoke.factory,
        runner: "smoke"
      }
    ));
    return [];
  }
}
