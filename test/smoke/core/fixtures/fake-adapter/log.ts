import fs from "node:fs";
import path from "node:path";

import type { FakeAdapterOptions } from "./types.ts";

export function recordCall(options: FakeAdapterOptions, extra: Record<string, unknown>) {
  if (!options.log) {
    return;
  }
  fs.mkdirSync(path.dirname(options.log), { recursive: true });
  fs.appendFileSync(
    options.log,
    `${JSON.stringify({
      adapter_id: options.id,
      mode: options.mode,
      command: options.command,
      argv: [options.command, ...options.commandArgs],
      cwd: process.cwd(),
      ...extra
    })}\n`,
    "utf8"
  );
}
