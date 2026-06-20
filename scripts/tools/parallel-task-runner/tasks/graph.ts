import { registerTaskId } from "./definition.ts";
import type { NormalizedTask } from "./definition.ts";

export function validateTaskGraph(taskList: readonly NormalizedTask[]): void {
  const ids = collectTaskIds(taskList);
  validateTaskDependencies(taskList, ids);
}

function collectTaskIds(taskList: readonly NormalizedTask[]): Set<string> {
  const ids = new Set<string>();
  for (const task of taskList) {
    registerTaskId(task.id, ids);
  }
  return ids;
}

function validateTaskDependencies(taskList: readonly NormalizedTask[], ids: Set<string>): void {
  for (const task of taskList) {
    for (const dependency of task.dependsOn) {
      if (!ids.has(dependency)) {
        throw new Error(`task ${task.id} depends on unknown task ${dependency}`);
      }
    }
  }
}
