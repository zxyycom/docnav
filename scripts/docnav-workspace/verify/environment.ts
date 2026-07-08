import { readJsonFile } from "../../tools/foundation/src/fs.ts";
import { isRecord } from "../../tools/foundation/src/type-guards.ts";
import type { CheckTask } from "../checks/index.ts";
import { resolveWorkspacePath } from "./paths.ts";

type StringMap = Record<string, string>;

export function environmentForCheck(check: CheckTask): NodeJS.ProcessEnv {
  return {
    ...process.env,
    ...readEnvFile(check.envFile),
    ...(check.env ?? {})
  };
}

function readEnvFile(envFile: string | undefined): StringMap {
  if (!envFile) {
    return {};
  }
  const envPath = resolveWorkspacePath(envFile);
  const parsed = readJsonFile(envPath);
  if (!isRecord(parsed)) {
    throw new Error(`env file must contain an object: ${envFile}`);
  }
  return Object.fromEntries(
    Object.entries(parsed).map(([key, value]) => [key, String(value)])
  );
}
