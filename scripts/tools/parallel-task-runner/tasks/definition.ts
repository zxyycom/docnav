export type TaskEnv = Record<string, string | undefined>;

export type StringList = string | readonly string[] | undefined;

export interface TaskDefinition {
  id: string;
  label?: string;
  type?: string;
  mutex?: StringList;
  dependsOn?: StringList;
  env?: TaskEnv;
  envFile?: string;
  tasks?: readonly TaskDefinition[];
  run?: (task: NormalizedTask) => unknown | Promise<unknown>;
  [key: string]: unknown;
}

export interface NormalizedTask extends TaskDefinition {
  label: string;
  type: string;
  mutex: string[];
  dependsOn: string[];
  tasks?: undefined;
}

export function normalizeTask(task: TaskDefinition): NormalizedTask {
  assertTaskObject(task);

  const { tasks: _tasks, ...rest } = task;
  return {
    label: task.id,
    type: "default",
    ...rest,
    mutex: normalizeStringList(task.mutex, "mutex"),
    dependsOn: normalizeStringList(task.dependsOn, "dependsOn")
  };
}

export function normalizeTaskList(taskList: readonly TaskDefinition[]): NormalizedTask[] {
  assertTaskList(taskList);
  return taskList.map(normalizeTask);
}

export function normalizeStringList(value: StringList, fieldName: string): string[] {
  if (value === undefined) {
    return [];
  }
  if (typeof value === "string") {
    return [value];
  }
  const maybeItems: unknown = value;
  if (!Array.isArray(maybeItems) || value.some((item) => item.length === 0)) {
    throw new Error(`task.${fieldName} must be a string or string array`);
  }
  return [...value];
}

export function assertTaskObject(task: unknown): asserts task is TaskDefinition {
  if (!task || typeof task !== "object") {
    throw new Error("task must be an object");
  }
  const value = task as Record<string, unknown>;
  if (typeof value.id !== "string" || value.id.trim().length === 0) {
    throw new Error("task.id must be a non-empty string");
  }
}

export function assertTaskList(taskList: readonly TaskDefinition[]): void {
  const maybeTaskList: unknown = taskList;
  if (!Array.isArray(maybeTaskList)) {
    throw new Error("task list must be an array");
  }
}

export function assertNonEmptyTaskList(taskList: readonly TaskDefinition[], message: string): void {
  const maybeTaskList: unknown = taskList;
  if (!Array.isArray(maybeTaskList) || taskList.length === 0) {
    throw new Error(message);
  }
}

export function registerTaskId(id: string, ids: Set<string>): void {
  if (ids.has(id)) {
    throw new Error(`duplicate task id: ${id}`);
  }
  ids.add(id);
}
