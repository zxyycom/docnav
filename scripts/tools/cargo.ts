import { isRecord } from "./types.ts";
import { processFailed, runProcessSync, writeProcessOutput } from "./process.ts";
import type { ProcessResult } from "./process.ts";

export type CargoBinarySpec = {
  binName: string;
  packageName: string;
};

export type BuildCargoExecutablesResult =
  | {
      executables: Map<string, string>;
      ok: true;
      stderr: string;
    }
  | {
      ok: false;
      reason: "process-failed";
      result: ProcessResult;
    }
  | {
      binName: string;
      ok: false;
      reason: "missing-executable";
    };

export type BuildCargoExecutablesFailure = Exclude<
  BuildCargoExecutablesResult,
  { ok: true }
>;

export function findCargoExecutable(output: string, binName: string): string | null {
  let executable: string | null = null;

  for (const line of output.split(/\r?\n/)) {
    if (line.trim().length === 0) {
      continue;
    }

    let message: unknown;
    try {
      const parsed: unknown = JSON.parse(line);
      message = parsed;
    } catch {
      continue;
    }

    if (!isRecord(message)) {
      continue;
    }
    const target = isRecord(message.target) ? message.target : null;
    const targetKinds = Array.isArray(target?.kind) ? target.kind : [];
    if (
      message.reason === "compiler-artifact" &&
      typeof message.executable === "string" &&
      target?.name === binName &&
      targetKinds.includes("bin")
    ) {
      executable = message.executable;
    }
  }

  return executable;
}

export function buildCargoExecutables({
  binaries,
  cwd
}: {
  binaries: readonly CargoBinarySpec[];
  cwd: string;
}): BuildCargoExecutablesResult {
  const packages = [...new Set(binaries.map((binary) => binary.packageName))];
  const cargoArgs = [
    "build",
    ...packages.flatMap((packageName) => ["-p", packageName]),
    ...binaries.flatMap((binary) => ["--bin", binary.binName]),
    "--message-format=json"
  ];
  const result = runProcessSync("cargo", cargoArgs, { cwd });

  if (processFailed(result)) {
    return {
      ok: false,
      reason: "process-failed",
      result
    };
  }

  const executables = new Map<string, string>();
  for (const binary of binaries) {
    const executable = findCargoExecutable(result.stdout ?? "", binary.binName);
    if (!executable) {
      return {
        binName: binary.binName,
        ok: false,
        reason: "missing-executable"
      };
    }
    executables.set(binary.binName, executable);
  }

  return {
    executables,
    ok: true,
    stderr: result.stderr
  };
}

export function reportCargoExecutableBuildFailure(
  failure: BuildCargoExecutablesFailure
): number {
  if (failure.reason === "process-failed") {
    writeProcessOutput(failure.result);
    if (failure.result.error) {
      console.error(failure.result.error.message);
    }
    return failure.result.status ?? 1;
  }

  console.error(`cargo build did not report a ${failure.binName} executable`);
  return 1;
}
