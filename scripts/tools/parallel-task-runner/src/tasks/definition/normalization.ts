import {
  assertTaskList,
  assertTaskObject
} from "./validation.ts";
import type { NormalizedTask, StringList, TaskDefinition } from "./types.ts";

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
    if (value.length === 0) {
      throw new Error(`task.${fieldName} must be a non-empty string or string array`);
    }
    return [value];
  }
  const maybeItems: unknown = value;
  if (!Array.isArray(maybeItems)) {
    throw new Error(`task.${fieldName} must be a string or string array`);
  }

  for (const [index, item] of maybeItems.entries()) {
    if (typeof item !== "string" || item.length === 0) {
      throw new Error(`task.${fieldName}[${index}] must be a non-empty string`);
    }
  }

  return [...value];
}
