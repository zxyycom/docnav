import type { SyncOptions } from "execa";

export const DEFAULT_PROCESS_MAX_BUFFER = 1024 * 1024 * 64;

export interface ProcessFailure extends Error {
  code?: number | string | null;
  signal?: NodeJS.Signals | null;
  status?: number | null;
  stderr?: Buffer | string;
  stdout?: Buffer | string;
}

export type ProcessResult = {
  error?: Error;
  signal: NodeJS.Signals | null;
  status: number | null;
  stderr: string;
  stdout: string;
};

export type RunProcessSyncOptions = {
  cwd?: string | URL;
  encoding?: SyncOptions["encoding"];
  env?: NodeJS.ProcessEnv;
  maxBuffer?: number;
  stdio?: SyncOptions["stdio"];
  timeout?: number;
  windowsHide?: boolean;
};

export type RunProcessOptions = {
  args?: string[];
  command: string;
  cwd?: string | URL;
  env?: NodeJS.ProcessEnv;
  label?: string;
  maxBuffer?: number;
  timeout?: number;
  windowsHide?: boolean;
};

export type ExecaResultLike = {
  code?: string;
  exitCode?: number;
  failed?: boolean;
  isMaxBuffer?: boolean;
  message?: string;
  originalMessage?: string;
  shortMessage?: string;
  signal?: NodeJS.Signals;
  stderr?: unknown;
  stdout?: unknown;
  timedOut?: boolean;
};
