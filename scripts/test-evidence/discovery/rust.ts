import path from "node:path";

import {
  astSourceRange,
  scanAstRule,
  unsupportedAstDiagnostics,
  type AstMatch
} from "../ast-scan.ts";
import { closeStaticAndRuntimeEntities } from "../closure.ts";
import {
  diagnostic,
  type StaticTestEntity,
  type TestEntity,
  type TestEvidenceDiagnostic
} from "../model.ts";
import type { SupportedRunnerProfile } from "../profile.ts";
import { enumerateCargoTests } from "./rust/cargo.ts";


export async function discoverRustEntities(options: {
  workspaceRoot: string;
  profile: SupportedRunnerProfile;
}): Promise<{
  entities: TestEntity[];
  diagnostics: TestEvidenceDiagnostic[];
}> {
  const staticResult = await scanRustStaticEntities(options);
  const runtimeResult = await enumerateCargoTests(options);
  const diagnostics = [
    ...staticResult.diagnostics,
    ...runtimeResult.diagnostics
  ];
  if (diagnostics.some(({ blocking }) => blocking)) {
    return {
      entities: [],
      diagnostics
    };
  }

  const closed = closeStaticAndRuntimeEntities({
    runner: "cargo",
    statics: staticResult.entities,
    runtime: runtimeResult.entities,
    createEntityKey: ({ target, selector }) => `cargo|${target}|${selector}`
  });
  return {
    entities: closed.entities,
    diagnostics: [...diagnostics, ...closed.diagnostics]
  };
}

async function scanRustStaticEntities(options: {
  workspaceRoot: string;
  profile: SupportedRunnerProfile;
}): Promise<{
  entities: StaticTestEntity[];
  diagnostics: TestEvidenceDiagnostic[];
}> {
  const scan = await scanRustSources(options);
  const candidates = rustStaticCandidates(scan.matches);
  return {
    entities: candidates.entities,
    diagnostics: [...scan.diagnostics, ...candidates.diagnostics]
  };
}

async function scanRustSources(options: {
  workspaceRoot: string;
  profile: SupportedRunnerProfile;
}): Promise<{
  matches: AstMatch[];
  diagnostics: TestEvidenceDiagnostic[];
}> {
  const ruleRoot = path.join(
    options.workspaceRoot,
    "scripts",
    "test-evidence",
    "rules"
  );
  const nativeScan = await scanAstRule({
    workspaceRoot: options.workspaceRoot,
    rulePath: path.join(ruleRoot, "rust-native-test.yml"),
    paths: options.profile.cargo.sourceRoots
  });
  const diagnostics = [...nativeScan.diagnostics];
  for (const ruleName of [
    "rust-unsupported-test-attribute.yml",
    "rust-unsupported-test-module-macro.yml"
  ]) {
    const scan = await scanAstRule({
      workspaceRoot: options.workspaceRoot,
      rulePath: path.join(ruleRoot, ruleName),
      paths: options.profile.cargo.sourceRoots
    });
    diagnostics.push(...scan.diagnostics);
    diagnostics.push(...unsupportedAstDiagnostics(scan.matches, "cargo"));
  }
  return {
    matches: nativeScan.matches,
    diagnostics
  };
}

function rustStaticCandidates(matches: readonly AstMatch[]): {
  entities: StaticTestEntity[];
  diagnostics: TestEvidenceDiagnostic[];
} {
  const entities: StaticTestEntity[] = [];
  const diagnostics: TestEvidenceDiagnostic[] = [];
  for (const match of matches) {
    const name = match.metaVariables.single.NAME?.text;
    if (!name) {
      diagnostics.push(diagnostic(
        "static-scan-failed",
        "static",
        "Rust native test rule did not capture NAME",
        {
          path: match.file,
          line: match.range.start.line + 1,
          runner: "cargo"
        }
      ));
      continue;
    }
    entities.push({
      identity: name,
      sourcePath: match.file,
      sourceRange: astSourceRange(match)
    });
  }
  return { entities, diagnostics };
}
