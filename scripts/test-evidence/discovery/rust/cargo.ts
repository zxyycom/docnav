import { runProcess } from "../../../tools/foundation/src/index.ts";
import { diagnostic, type RuntimeTestEntity, type TestEvidenceDiagnostic } from "../../model.ts";
import type { SupportedRunnerProfile } from "../../profile.ts";
import { processFailureMessage, runMiseCommand } from "../../runner-process.ts";

type CargoDiscoveryOptions = { workspaceRoot: string; profile: SupportedRunnerProfile };
type CargoEnumerationResult = { entities: RuntimeTestEntity[]; diagnostics: TestEvidenceDiagnostic[] };
type CargoMetadataResult = { packageNames?: Map<string, string>; diagnostics: TestEvidenceDiagnostic[] };
type CargoBuildResult = { artifacts?: CargoArtifact[]; diagnostics: TestEvidenceDiagnostic[] };
type CargoArtifact = { packageName: string; kind: string; targetName: string; executable: string };

type CargoArtifactMessage = {
  package_id: string;
  executable: string;
  target: Record<string, unknown> & { name: string; kind: unknown[] };
};

export async function enumerateCargoTests(options: CargoDiscoveryOptions): Promise<CargoEnumerationResult> {
  const metadata = await loadCargoPackageNames(options.workspaceRoot);
  if (!metadata.packageNames) {
    return {
      entities: [],
      diagnostics: metadata.diagnostics
    };
  }

  const build = await buildCargoArtifacts(options, metadata.packageNames);
  if (!build.artifacts) {
    return {
      entities: [],
      diagnostics: build.diagnostics
    };
  }

  const listed = await enumerateCargoArtifactTests(
    options.workspaceRoot,
    build.artifacts
  );
  const doctestDiagnostics = await diagnoseCargoDoctests(options.workspaceRoot);
  return {
    entities: listed.entities,
    diagnostics: [...listed.diagnostics, ...doctestDiagnostics]
  };
}

async function loadCargoPackageNames(workspaceRoot: string): Promise<CargoMetadataResult> {
  const result = await runMiseCommand({
    workspaceRoot,
    command: "cargo",
    args: ["metadata", "--locked", "--format-version=1", "--no-deps"],
    label: "cargo metadata"
  });
  if (result.status !== 0) {
    return {
      diagnostics: [
        diagnostic(
          "runner-metadata-failed",
          "runner",
          processFailureMessage(result, "cargo metadata"),
          { runner: "cargo" }
        )
      ]
    };
  }

  try {
    return {
      packageNames: parseCargoPackageNames(result.stdout),
      diagnostics: []
    };
  } catch (error) {
    return {
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
}

async function buildCargoArtifacts(
  options: CargoDiscoveryOptions,
  packageNames: Map<string, string>
): Promise<CargoBuildResult> {
  const result = await runMiseCommand({
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
  if (result.status !== 0) {
    return {
      diagnostics: [
        diagnostic(
          "runner-build-failed",
          "runner",
          processFailureMessage(result, "cargo test --no-run"),
          { runner: "cargo" }
        )
      ]
    };
  }

  try {
    return {
      artifacts: parseCargoArtifacts(
        result.stdout,
        packageNames,
        new Set(options.profile.cargo.targetKinds)
      ),
      diagnostics: []
    };
  } catch (error) {
    return {
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
}

async function enumerateCargoArtifactTests(
  workspaceRoot: string,
  artifacts: readonly CargoArtifact[]
): Promise<CargoEnumerationResult> {
  const entities: RuntimeTestEntity[] = [];
  const diagnostics: TestEvidenceDiagnostic[] = [];
  for (const artifact of artifacts) {
    const label =
      `cargo test list ${artifact.packageName}:${artifact.kind}:${artifact.targetName}`;
    const result = await runProcess({
      command: artifact.executable,
      args: ["--list", "--format", "terse"],
      cwd: workspaceRoot,
      label
    });
    if (result.status !== 0) {
      diagnostics.push(diagnostic(
        "runner-list-failed",
        "runner",
        processFailureMessage(result, label),
        {
          runner: "cargo",
          target: cargoTarget(artifact)
        }
      ));
      continue;
    }
    for (const selector of parseLibtestList(result.stdout)) {
      entities.push({
        identity: selector.split("::").at(-1) ?? selector,
        target: cargoTarget(artifact),
        selector
      });
    }
  }
  return { entities, diagnostics };
}

async function diagnoseCargoDoctests(
  workspaceRoot: string
): Promise<TestEvidenceDiagnostic[]> {
  const result = await runMiseCommand({
    workspaceRoot,
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
  if (result.status !== 0) {
    return [
      diagnostic(
        "runner-list-failed",
        "runner",
        processFailureMessage(result, "cargo doctest list"),
        {
          runner: "cargo",
          target: "doctest"
        }
      )
    ];
  }
  return parseLibtestList(result.stdout).map((selector) => diagnostic(
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
    const artifact = normalizeCargoArtifactMessage(
      JSON.parse(line) as unknown,
      packageNames,
      targetKinds
    );
    if (artifact) {
      artifacts.set(cargoTarget(artifact), artifact);
    }
  }
  return [...artifacts.values()].sort((left, right) => (
    cargoTarget(left).localeCompare(cargoTarget(right))
  ));
}

function normalizeCargoArtifactMessage(
  value: unknown,
  packageNames: Map<string, string>,
  targetKinds: Set<string>
): CargoArtifact | null {
  if (!isCargoArtifactMessage(value)) {
    return null;
  }
  const kind = value.target.kind.find((candidate) => (
    typeof candidate === "string" && targetKinds.has(candidate)
  ));
  if (typeof kind !== "string") {
    return null;
  }
  const packageName = packageNames.get(value.package_id);
  if (!packageName) {
    throw new Error(`unknown Cargo package id ${value.package_id}`);
  }
  return {
    packageName,
    kind,
    targetName: value.target.name,
    executable: value.executable
  };
}

function isCargoArtifactMessage(value: unknown): value is CargoArtifactMessage {
  return (
    isRecord(value) &&
    value.reason === "compiler-artifact" &&
    typeof value.package_id === "string" &&
    typeof value.executable === "string" &&
    isRecord(value.profile) &&
    value.profile.test === true &&
    isRecord(value.target) &&
    typeof value.target.name === "string" &&
    Array.isArray(value.target.kind)
  );
}

function cargoTarget(artifact: Pick<CargoArtifact, "packageName" | "kind" | "targetName">): string {
  return `${artifact.packageName}:${artifact.kind}:${artifact.targetName}`;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
