import {
  buildCargoExecutables,
  reportCargoExecutableBuildFailure
} from "../../tools/cargo.ts";
import type { BinarySpec } from "./specs.ts";

export function buildCargoBins(cwd: string, binaries: BinarySpec[], quiet: boolean): Map<string, string> {
  const result = buildCargoExecutables({ binaries, cwd });

  if (!result.ok) {
    process.exit(reportCargoExecutableBuildFailure(result));
  }

  if (result.stderr && !quiet) {
    process.stderr.write(result.stderr);
  }

  return result.executables;
}

export function environmentWithCargoBins(
  binaries: BinarySpec[],
  executables: ReadonlyMap<string, string>
): NodeJS.ProcessEnv {
  const env = { ...process.env };
  for (const binary of binaries) {
    const executable = executables.get(binary.binName);
    if (!executable) {
      console.error(`cargo build did not report a ${binary.binName} executable`);
      process.exit(1);
    }
    env[binary.envName] = executable;
  }
  return env;
}
