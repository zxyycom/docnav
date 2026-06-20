import {
  normalizeStringList,
  normalizeTask
} from "./definition/normalization.ts";
import {
  assertNonEmptyTaskList,
  assertTaskList,
  assertTaskObject,
  registerTaskId
} from "./definition/validation.ts";
import type { NormalizedTask, TaskDefinition, TaskEnv } from "./definition/types.ts";

export { normalizeTask, normalizeTaskList } from "./definition/normalization.ts";
export type { NormalizedTask, StringList, TaskDefinition, TaskEnv } from "./definition/types.ts";

interface InheritedTaskState {
  type: string;
  mutex: string[];
  dependsOn: string[];
  env?: TaskEnv;
  envFile?: string;
}

interface ExpandTaskState {
  ids: Set<string>;
  groupLeafIds: Map<string, string[]>;
  leafTasks: NormalizedTask[];
}

const DEFAULT_INHERITED_TASK_STATE: InheritedTaskState = {
  type: "default",
  mutex: [],
  dependsOn: [],
  env: undefined,
  envFile: undefined
};

export function expandTasks(taskList: readonly TaskDefinition[]): NormalizedTask[] {
  assertTaskList(taskList);

  const state: ExpandTaskState = {
    ids: new Set<string>(),
    groupLeafIds: new Map<string, string[]>(),
    leafTasks: []
  };

  for (const task of taskList) {
    expandTask(task, DEFAULT_INHERITED_TASK_STATE, state);
  }

  return state.leafTasks.map((task) => ({
    ...task,
    dependsOn: resolveGroupDependencies(task.dependsOn, state.groupLeafIds, task.id)
  }));
}

function expandTask(task: TaskDefinition, inherited: InheritedTaskState, state: ExpandTaskState): void {
  assertTaskObject(task);
  registerTaskId(task.id, state.ids);

  const nextInherited = inheritedTaskState(task, inherited);

  if (task.tasks !== undefined) {
    expandTaskGroup(task.id, task.tasks, nextInherited, state);
    return;
  }

  state.leafTasks.push(normalizeTask(leafTaskDefinition(task, nextInherited)));
}

function inheritedTaskState(task: TaskDefinition, inherited: InheritedTaskState): InheritedTaskState {
  const taskMutex = normalizeStringList(task.mutex, "mutex");
  const taskDependsOn = normalizeStringList(task.dependsOn, "dependsOn");

  return {
    type: task.type ?? inherited.type,
    mutex: [...inherited.mutex, ...taskMutex],
    dependsOn: [...inherited.dependsOn, ...taskDependsOn],
    env: mergeEnv(inherited.env, task.env),
    envFile: task.envFile ?? inherited.envFile
  };
}

function expandTaskGroup(
  taskId: string,
  childTasks: readonly TaskDefinition[],
  inherited: InheritedTaskState,
  state: ExpandTaskState
): void {
  assertNonEmptyTaskList(childTasks, "task.tasks must be a non-empty array");

  const startIndex = state.leafTasks.length;
  for (const child of childTasks) {
    expandTask(child, inherited, state);
  }

  state.groupLeafIds.set(
    taskId,
    state.leafTasks.slice(startIndex).map((leaf) => leaf.id)
  );
}

function leafTaskDefinition(task: TaskDefinition, inherited: InheritedTaskState): TaskDefinition {
  const {
    dependsOn: _dependsOn,
    env: _env,
    envFile: _envFile,
    mutex: _mutex,
    tasks: _tasks,
    type: _type,
    ...rest
  } = task;
  const leaf: TaskDefinition = {
    type: inherited.type,
    ...rest,
    mutex: inherited.mutex,
    dependsOn: inherited.dependsOn
  };
  if (inherited.env !== undefined) {
    leaf.env = inherited.env;
  }
  if (inherited.envFile !== undefined) {
    leaf.envFile = inherited.envFile;
  }
  return leaf;
}

function mergeEnv(parentEnv: TaskEnv | undefined, taskEnv: TaskEnv | undefined): TaskEnv | undefined {
  if (parentEnv === undefined) {
    return taskEnv;
  }
  if (taskEnv === undefined) {
    return parentEnv;
  }
  return {
    ...parentEnv,
    ...taskEnv
  };
}

function resolveGroupDependencies(dependsOn: readonly string[], groupLeafIds: Map<string, string[]>, taskId: string): string[] {
  const resolved: string[] = [];
  const seen = new Set<string>();
  for (const dependency of dependsOn) {
    const dependencies = groupLeafIds.get(dependency) ?? [dependency];
    for (const id of dependencies) {
      if (id !== taskId && !seen.has(id)) {
        resolved.push(id);
        seen.add(id);
      }
    }
  }
  return resolved;
}
