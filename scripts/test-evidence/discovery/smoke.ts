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
import { closeStaticAndRuntimeEntities } from "../closure.ts";
import {
  diagnostic,
  type RuntimeTestEntity,
  type StaticTestEntity,
  type TestEntity,
  type TestEvidenceDiagnostic
} from "../model.ts";
import {
  SUPPORTED_SMOKE_FACTORY,
  type SupportedRunnerProfile
} from "../profile.ts";

type SmokeDiscoveryOptions = {
  workspaceRoot: string;
  profile: SupportedRunnerProfile;
};

export async function discoverSmokeEntities(
  options: SmokeDiscoveryOptions
): Promise<{
  entities: TestEntity[];
  diagnostics: TestEvidenceDiagnostic[];
}> {
  const diagnostics: TestEvidenceDiagnostic[] = [];
  if (options.profile.smoke.factory !== SUPPORTED_SMOKE_FACTORY) {
    return {
      entities: [],
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
  const statics = createStaticCandidates(matches, diagnostics);
  const runtime = createRuntimeEntities(options.profile, diagnostics);

  if (diagnostics.some(({ blocking }) => blocking)) {
    return {
      entities: [],
      diagnostics
    };
  }
  const closed = closeStaticAndRuntimeEntities({
    runner: "smoke",
    statics,
    runtime,
    createEntityKey: ({ target, selector }) => `smoke|${target}|${selector}`
  });
  return {
    entities: closed.entities,
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
  matches: AstMatch[],
  diagnostics: TestEvidenceDiagnostic[]
): StaticTestEntity[] {
  const statics: StaticTestEntity[] = [];
  for (const match of matches) {
    const candidate = createStaticCandidate(match, diagnostics);
    if (candidate) {
      statics.push(candidate);
    }
  }
  return statics;
}

function createStaticCandidate(
  match: AstMatch,
  diagnostics: TestEvidenceDiagnostic[]
): StaticTestEntity | null {
  const id = match.metaVariables.single.ID?.text;
  if (!id) {
    diagnostics.push(diagnostic(
      "static-scan-failed",
      "static",
      "smoke native leaf rule did not capture ID",
      {
        path: match.file,
        line: match.range.start.line + 1,
        runner: "smoke"
      }
    ));
    return null;
  }
  return {
    identity: id,
    sourcePath: match.file,
    sourceRange: astSourceRange(match)
  };
}

function createRuntimeEntities(
  profile: SupportedRunnerProfile,
  diagnostics: TestEvidenceDiagnostic[]
): RuntimeTestEntity[] {
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
