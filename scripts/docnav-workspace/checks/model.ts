import type {
  NormalizedTask,
  StringList,
  TaskEnv
} from "../../tools/parallel-task-runner/src/index.ts";

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
    description: "required non-quality checks plus full quality check, smoke, Rust, and OpenSpec gates"
  }
});

type CheckDefinitionBase = {
  dependsOn?: StringList;
  env?: TaskEnv;
  envFile?: string;
  id: string;
  label?: string;
  mutex?: StringList;
  type?: string;
};

type CheckLeafDefinition = CheckDefinitionBase & {
  allowOutput?: RegExp[];
  args?: string[];
  command: string;
  ignoreOutput?: RegExp[];
  tasks?: never;
  warningOutput?: RegExp[];
};

type CheckGroupDefinition = CheckDefinitionBase & {
  allowOutput?: never;
  args?: never;
  command?: never;
  ignoreOutput?: never;
  tasks: readonly [CheckDefinition, ...CheckDefinition[]];
  warningOutput?: never;
};

export type CheckDefinition = CheckLeafDefinition | CheckGroupDefinition;

export interface CheckTask extends NormalizedTask {
  allowOutput?: RegExp[];
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
