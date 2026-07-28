import path from "node:path";

import { runProcess } from "../../tools/foundation/src/index.ts";
import {
  astSourceRange,
  scanAstRule,
  unsupportedAstDiagnostics
} from "../ast-scan.ts";
import { closeStaticAndRuntimeEntities } from "../closure.ts";
import {
  diagnostic,
  type RuntimeTestEntity,
  type StaticTestEntity,
  type TestEntity,
  type TestEvidenceDiagnostic
} from "../model.ts";
import type { SupportedRunnerProfile } from "../profile.ts";
import {
  processFailureMessage,
  runMiseCommand
} from "../runner-process.ts";

type CargoArtifact = {
  packageName: string;
  kind: string;
  targetName: string;
  executable: string;
};

export async function discoverRustEntities(options: {
  workspaceRoot: string;
  profile: SupportedRunnerProfile;
}): Promise<{
  entities: TestEntity[];
  diagnostics: TestEvidenceDiagnostic[];
}> {
  const diagnostics: TestEvidenceDiagnostic[] = [];
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
  diagnostics.push(...nativeScan.diagnostics);

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

  const statics: StaticTestEntity[] = [];
  for (const match of nativeScan.matches) {
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
    statics.push({
      identity: name,
      sourcePath: match.file,
      sourceRange: astSourceRange(match)
    });
  }

  const runtimeResult = await enumerateCargoTests(options);
  diagnostics.push(...runtimeResult.diagnostics);
  if (diagnostics.some(({ blocking }) => blocking)) {
    return {
      entities: [],
      diagnostics
    };
  }

  const closed = closeStaticAndRuntimeEntities({
    runner: "cargo",
    statics,
    runtime: runtimeResult.entities,
    createEntityKey: ({ target, selector }) => `cargo|${target}|${selector}`
  });
  return {
    entities: closed.entities,
    diagnostics: [...diagnostics, ...closed.diagnostics]
  };
}

async function enumerateCargoTests(options: {
  workspaceRoot: string;
  profile: SupportedRunnerProfile;
}): Promise<{
  entities: RuntimeTestEntity[];
  diagnostics: TestEvidenceDiagnostic[];
}> {
  const diagnostics: TestEvidenceDiagnostic[] = [];
  const metadataResult = await runMiseCommand({
    workspaceRoot: options.workspaceRoot,
    command: "cargo",
    args: ["metadata", "--locked", "--format-version=1", "--no-deps"],
    label: "cargo metadata"
  });
  if (metadataResult.status !== 0) {
    return {
      entities: [],
      diagnostics: [
        diagnostic(
          "runner-metadata-failed",
          "runner",
          processFailureMessage(metadataResult, "cargo metadata"),
          { runner: "cargo" }
        )
      ]
    };
  }

  let packageNames;
  try {
    packageNames = parseCargoPackageNames(metadataResult.stdout);
  } catch (error) {
    return {
      entities: [],
      diagnostics: [
        diagnostic(
          "runner-report-invalid",
          "runner",
          `cargo metadata report is malformed: ${error instanceof Error ? error.message : String(error)}`,
          { runner: "cargo" }
        )
      ]
    };
  }

  const buildResult = await runMiseCommand({
    workspaceRoot: options.workspaceRoot,
    command: "cargo",
    args: [
      "test",
      "--locked",
      "--workspace",
      "--no-run",
      "--message-format=json"
    ],
    label: "cargo test --no-run"
  });
  if (buildResult.status !== 0) {
    return {
      entities: [],
      diagnostics: [
        diagnostic(
          "runner-build-failed",
          "runner",
          processFailureMessage(buildResult, "cargo test --no-run"),
          { runner: "cargo" }
        )
      ]
    };
  }

  let artifacts;
  try {
    artifacts = parseCargoArtifacts(
      buildResult.stdout,
      packageNames,
      new Set(options.profile.cargo.targetKinds)
    );
  } catch (error) {
    return {
      entities: [],
      diagnostics: [
        diagnostic(
          "runner-report-invalid",
          "runner",
          `cargo compiler artifact report is malformed: ${error instanceof Error ? error.message : String(error)}`,
          { runner: "cargo" }
        )
      ]
    };
  }

  const entities: RuntimeTestEntity[] = [];
  for (const artifact of artifacts) {
    const listResult = await runProcess({
      command: artifact.executable,
      args: ["--list", "--format", "terse"],
      cwd: options.workspaceRoot,
      label: `cargo test list ${artifact.packageName}:${artifact.kind}:${artifact.targetName}`
    });
    if (listResult.status !== 0) {
      diagnostics.push(diagnostic(
        "runner-list-failed",
        "runner",
        processFailureMessage(
          listResult,
          `cargo test list ${artifact.packageName}:${artifact.kind}:${artifact.targetName}`
        ),
        {
          runner: "cargo",
          target: cargoTarget(artifact)
        }
      ));
      continue;
    }
    for (const selector of parseLibtestList(listResult.stdout)) {
      entities.push({
        identity: selector.split("::").at(-1) ?? selector,
        target: cargoTarget(artifact),
        selector
      });
    }
  }

  const doctestResult = await runMiseCommand({
    workspaceRoot: options.workspaceRoot,
    command: "cargo",
    args: [
      "test",
      "--locked",
      "--workspace",
      "--doc",
      "--",
      "--list",
      "--format",
      "terse"
    ],
    label: "cargo doctest list"
  });
  if (doctestResult.status !== 0) {
    diagnostics.push(diagnostic(
      "runner-list-failed",
      "runner",
      processFailureMessage(doctestResult, "cargo doctest list"),
      {
        runner: "cargo",
        target: "doctest"
      }
    ));
  } else {
    for (const selector of parseLibtestList(doctestResult.stdout)) {
      diagnostics.push(diagnostic(
        "unsupported-entity-shape",
        "runner",
        `Cargo doctest ${selector} is runtime-visible but profile v1 has no static doctest adapter`,
        {
          runner: "cargo",
          target: "doctest",
          selector
        }
      ));
    }
  }

  return {
    entities,
    diagnostics
  };
}

export function parseLibtestList(stdout: string): string[] {
  return stdout
    .split(/\r?\n/u)
    .map((line) => line.trim())
    .filter((line) => line.endsWith(": test"))
    .map((line) => line.slice(0, -": test".length));
}

function parseCargoPackageNames(stdout: string): Map<string, string> {
  const value: unknown = JSON.parse(stdout);
  if (!isRecord(value) || !Array.isArray(value.packages)) {
    throw new Error("packages array is missing");
  }
  const result = new Map<string, string>();
  for (const packageValue of value.packages) {
    if (
      !isRecord(packageValue) ||
      typeof packageValue.id !== "string" ||
      typeof packageValue.name !== "string"
    ) {
      throw new Error("package identity is invalid");
    }
    result.set(packageValue.id, packageValue.name);
  }
  return result;
}

function parseCargoArtifacts(
  stdout: string,
  packageNames: Map<string, string>,
  targetKinds: Set<string>
): CargoArtifact[] {
  const artifacts = new Map<string, CargoArtifact>();
  for (const line of stdout.split(/\r?\n/u)) {
    if (line.trim() === "") {
      continue;
    }
    const value: unknown = JSON.parse(line);
    if (!isRecord(value) || value.reason !== "compiler-artifact") {
      continue;
    }
    if (
      typeof value.package_id !== "string" ||
      typeof value.executable !== "string" ||
      !isRecord(value.profile) ||
      value.profile.test !== true ||
      !isRecord(value.target) ||
      typeof value.target.name !== "string" ||
      !isUnknownArray(value.target.kind)
    ) {
      continue;
    }
    const kind = value.target.kind.find((candidate) => (
      typeof candidate === "string" && targetKinds.has(candidate)
    ));
    if (typeof kind !== "string") {
      continue;
    }
    const packageName = packageNames.get(value.package_id);
    if (!packageName) {
      throw new Error(`unknown Cargo package id ${value.package_id}`);
    }
    const artifact = {
      packageName,
      kind,
      targetName: value.target.name,
      executable: value.executable
    };
    artifacts.set(cargoTarget(artifact), artifact);
  }
  return [...artifacts.values()].sort((left, right) => (
    cargoTarget(left).localeCompare(cargoTarget(right))
  ));
}

function cargoTarget(artifact: Pick<CargoArtifact, "packageName" | "kind" | "targetName">): string {
  return `${artifact.packageName}:${artifact.kind}:${artifact.targetName}`;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isUnknownArray(value: unknown): value is unknown[] {
  return Array.isArray(value);
}
