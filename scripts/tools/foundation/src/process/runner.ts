import { execa, execaSync } from "execa";

import { plainTextProcessEnv } from "./env.ts";
import { toProcessResult } from "./result.ts";
import {
  DEFAULT_PROCESS_MAX_BUFFER,
  type ProcessResult,
  type RunProcessOptions,
  type RunProcessSyncOptions
} from "./types.ts";

export function runProcessSync(
  command: string,
  args: string[],
  options: RunProcessSyncOptions = {}
): ProcessResult {
  const result = execaSync(command, args, {
    ...options,
    encoding: options.encoding ?? "utf8",
    env: plainTextProcessEnv(options.env),
    maxBuffer: options.maxBuffer ?? DEFAULT_PROCESS_MAX_BUFFER,
    reject: false,
    stripFinalNewline: false,
    windowsHide: options.windowsHide ?? true
  });
  return toProcessResult(result, command);
}

export function runProcess(options: RunProcessOptions): Promise<ProcessResult> {
  const {
    args = [],
    command,
    cwd,
    env,
    label = command,
    maxBuffer = DEFAULT_PROCESS_MAX_BUFFER,
    timeout,
    windowsHide = true
  } = options;

  return execa(command, args, {
    cwd,
    env: plainTextProcessEnv(env),
    maxBuffer,
    reject: false,
    stripFinalNewline: false,
    timeout,
    windowsHide
  }).then((result) => toProcessResult(result, label));
}
