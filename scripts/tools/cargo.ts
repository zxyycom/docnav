import { isRecord } from "./types.ts";

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
