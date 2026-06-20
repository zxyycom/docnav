import type { TaskDefinition } from "./types.ts";

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
