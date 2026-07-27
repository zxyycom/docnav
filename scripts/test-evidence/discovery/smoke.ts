import path from "node:path";

import {
  prepareSmokeTasks,
  selectSmokeTasks
} from "../../../test/tools/smoke-harness.ts";
import { createCoreSmokeTasks } from "../../../test/smoke/core/profile.ts";
import {
  astSourceFingerprint,
  astSourceRange,
  scanAstRule,
  unsupportedAstDiagnostics
} from "../ast-scan.ts";
import { closeStaticAndRuntimeEntries } from "../closure.ts";
import {
  diagnostic,
  type NativeTestEntry,
  type RuntimeTestEntry,
  type StaticTestCandidate,
  type TestEvidenceDiagnostic
} from "../model.ts";
import type { SupportedRunnerProfile } from "../profile.ts";

const CORE_SMOKE_FACTORY = "test/smoke/core/profile.ts";

export async function discoverSmokeEntries(options: {
  workspaceRoot: string;
  profile: SupportedRunnerProfile;
}): Promise<{
  entries: NativeTestEntry[];
  diagnostics: TestEvidenceDiagnostic[];
}> {
  const diagnostics: TestEvidenceDiagnostic[] = [];
  if (options.profile.smoke.factory !== CORE_SMOKE_FACTORY) {
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
  diagnostics.push(...nativeScan.diagnostics);
  const unsupportedScan = await scanAstRule({
    workspaceRoot: options.workspaceRoot,
    rulePath: path.join(ruleRoot, "smoke-unsupported-dynamic.yml"),
    paths: options.profile.smoke.sourceRoots
  });
  diagnostics.push(...unsupportedScan.diagnostics);
  diagnostics.push(...unsupportedAstDiagnostics(
    unsupportedScan.matches,
    "smoke"
  ));

  const statics: StaticTestCandidate[] = [];
  for (const match of nativeScan.matches) {
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
      continue;
    }
    statics.push({
      identity: id,
      sourcePath: match.file,
      sourceRange: astSourceRange(match),
      sourceFingerprint: astSourceFingerprint(match)
    });
  }

  let runtime: RuntimeTestEntry[] = [];
  try {
    const prepared = prepareSmokeTasks(createCoreSmokeTasks());
    runtime = prepared.map((task) => {
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
        target: `${options.profile.smoke.id}:${reportId}`,
        selector: task.id
      };
    });
  } catch (error) {
    diagnostics.push(diagnostic(
      "runner-list-failed",
      "runner",
      `smoke task expansion failed: ${error instanceof Error ? error.message : String(error)}`,
      {
        path: options.profile.smoke.factory,
        runner: "smoke"
      }
    ));
  }

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
