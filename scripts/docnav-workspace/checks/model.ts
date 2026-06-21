import type { NormalizedTask, TaskDefinition } from "../../tools/parallel-task-runner/index.ts";

export const PROFILE_REQUIRED = "required";
export const PROFILE_FULL = "full";

export type Profile = typeof PROFILE_REQUIRED | typeof PROFILE_FULL;
export type CheckStatus = "passed" | "warning" | "failed";

export const profiles = Object.freeze({
  [PROFILE_REQUIRED]: {
    label: "required",
    description: "fast deterministic checks and quick quality check for routine development"
  },
  [PROFILE_FULL]: {
    label: "full",
    description: "required checks plus full quality check, smoke, Rust, and OpenSpec gates"
  }
});

export type CheckDefinition = TaskDefinition & {
  args?: string[];
  command?: string;
  ignoreOutput?: RegExp[];
  tasks?: readonly CheckDefinition[];
  warningOutput?: RegExp[];
};

export interface CheckTask extends NormalizedTask {
  args: string[];
  command: string;
  ignoreOutput: RegExp[];
  reportId?: string;
  reportLabel?: string;
  warningOutput: RegExp[];
}

export interface CheckReportRef {
  id: string;
  label: string;
}
