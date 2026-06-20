import { parsePositiveInteger } from "../args.ts";

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

interface RunParallelTaskOptions<TResult> {
  prepareTasks?: (taskList: readonly TaskDefinition[]) => NormalizedTask[];
  execute?: (task: NormalizedTask) => TResult | Promise<TResult>;
  onStart?: (task: NormalizedTask) => unknown | Promise<unknown>;
  onComplete?: (result: TResult, task: NormalizedTask) => unknown | Promise<unknown>;
  concurrency?: string | number | null;
}

interface StartTaskOptions<TResult> {
  task: NormalizedTask;
  execute: (task: NormalizedTask) => TResult | Promise<TResult>;
  onStart: (task: NormalizedTask) => unknown | Promise<unknown>;
  onComplete: (result: TResult, task: NormalizedTask) => unknown | Promise<unknown>;
  completedIds: Set<string>;
  runningMutexes: Set<string>;
  results: TResult[];
  onSettled: () => void;
  onError: (error: unknown) => void;
  isSettled: () => boolean;
}

interface ParallelTaskScheduler<TResult> {
  pending: NormalizedTask[];
  concurrency: number;
  execute: (task: NormalizedTask) => TResult | Promise<TResult>;
  onStart: (task: NormalizedTask) => unknown | Promise<unknown>;
  onComplete: (result: TResult, task: NormalizedTask) => unknown | Promise<unknown>;
  completedIds: Set<string>;
  runningMutexes: Set<string>;
  results: TResult[];
  activeCount: number;
  settled: boolean;
}

export async function runParallelTasks<TResult = unknown>(
  taskList: readonly TaskDefinition[],
  options: RunParallelTaskOptions<TResult> = {}
): Promise<TResult[]> {
  const prepareTasks = options.prepareTasks ?? normalizeTaskList;
  const execute = options.execute ?? (executeTask as (task: NormalizedTask) => TResult | Promise<TResult>);
  const onStart = options.onStart ?? noop;
  const onComplete = options.onComplete ?? noop;
  const pending = prepareTasks(taskList);
  const concurrency = resolveConcurrency(options.concurrency, pending.length);
  validateTaskGraph(pending);

  const scheduler = createParallelTaskScheduler({
    pending,
    concurrency,
    execute,
    onStart,
    onComplete
  });

  await runTaskScheduler(scheduler);
  return scheduler.results;
}

function resolveConcurrency(value: string | number | null | undefined, taskCount: number): number {
  if (value === undefined || value === null) {
    return taskCount;
  }

  return parsePositiveInteger(value, "task concurrency");
}

function createParallelTaskScheduler<TResult>({
  pending,
  concurrency,
  execute,
  onStart,
  onComplete
}: Pick<ParallelTaskScheduler<TResult>, "pending" | "concurrency" | "execute" | "onStart" | "onComplete">): ParallelTaskScheduler<TResult> {
  return {
    pending,
    concurrency,
    execute,
    onStart,
    onComplete,
    completedIds: new Set<string>(),
    runningMutexes: new Set<string>(),
    results: [],
    activeCount: 0,
    settled: false
  };
}

async function runTaskScheduler<TResult>(scheduler: ParallelTaskScheduler<TResult>): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    const finishIfDone = () => {
      if (completeSchedulerIfDone(scheduler)) {
        resolve();
      }
    };

    const fail = (error: unknown) => {
      if (failScheduler(scheduler)) {
        reject(error);
      }
    };

    const schedule = () => {
      scheduleReadyTasks(scheduler, {
        onSettled: () => {
          scheduler.activeCount -= 1;
          schedule();
          finishIfDone();
        },
        onError: fail,
        isSettled: () => scheduler.settled
      });

      if (scheduler.activeCount === 0 && scheduler.pending.length > 0) {
        fail(new Error(`unable to schedule tasks; unresolved dependencies or cycle: ${describePendingTasks(scheduler.pending, scheduler.completedIds)}`));
        return;
      }

      finishIfDone();
    };

    schedule();
  });
}

function scheduleReadyTasks<TResult>(
  scheduler: ParallelTaskScheduler<TResult>,
  callbacks: Pick<StartTaskOptions<TResult>, "onSettled" | "onError" | "isSettled">
): void {
  while (scheduler.activeCount < scheduler.concurrency) {
    const nextIndex = scheduler.pending.findIndex((task) => canRunTask(task, scheduler.completedIds, scheduler.runningMutexes));
    if (nextIndex === -1) {
      break;
    }

    const [task] = scheduler.pending.splice(nextIndex, 1);
    startTask({
      task,
      execute: scheduler.execute,
      onStart: scheduler.onStart,
      onComplete: scheduler.onComplete,
      completedIds: scheduler.completedIds,
      runningMutexes: scheduler.runningMutexes,
      results: scheduler.results,
      ...callbacks
    });
    scheduler.activeCount += 1;
  }
}

function completeSchedulerIfDone<TResult>(scheduler: ParallelTaskScheduler<TResult>): boolean {
  if (scheduler.settled || scheduler.pending.length > 0 || scheduler.activeCount > 0) {
    return false;
  }
  scheduler.settled = true;
  return true;
}

function failScheduler<TResult>(scheduler: ParallelTaskScheduler<TResult>): boolean {
  if (scheduler.settled) {
    return false;
  }
  scheduler.settled = true;
  return true;
}

export function expandTasks(taskList: readonly TaskDefinition[]): NormalizedTask[] {
  const maybeTaskList: unknown = taskList;
  if (!Array.isArray(maybeTaskList)) {
    throw new Error("task list must be an array");
  }

  const ids = new Set<string>();
  const groupLeafIds = new Map<string, string[]>();
  const leafTasks: NormalizedTask[] = [];

  for (const task of taskList) {
    expandTask(task, {
      type: "default",
      mutex: [],
      dependsOn: [],
      env: undefined,
      envFile: undefined
    }, {
      ids,
      groupLeafIds,
      leafTasks
    });
  }

  return leafTasks.map((task) => ({
    ...task,
    dependsOn: resolveGroupDependencies(task.dependsOn, groupLeafIds, task.id)
  }));
}

export function normalizeTask(task: TaskDefinition): NormalizedTask {
  if (!task || typeof task !== "object") {
    throw new Error("task must be an object");
  }
  if (typeof task.id !== "string" || task.id.trim().length === 0) {
    throw new Error("task.id must be a non-empty string");
  }

  const { tasks: _tasks, ...rest } = task;
  return {
    label: task.id,
    type: "default",
    ...rest,
    mutex: normalizeStringList(task.mutex, "mutex"),
    dependsOn: normalizeStringList(task.dependsOn, "dependsOn")
  };
}

export function validateTaskGraph(taskList: readonly NormalizedTask[]): void {
  const ids = new Set<string>();
  for (const task of taskList) {
    if (ids.has(task.id)) {
      throw new Error(`duplicate task id: ${task.id}`);
    }
    ids.add(task.id);
  }

  for (const task of taskList) {
    for (const dependency of task.dependsOn) {
      if (!ids.has(dependency)) {
        throw new Error(`task ${task.id} depends on unknown task ${dependency}`);
      }
    }
  }
}

function expandTask(task: TaskDefinition, inherited: InheritedTaskState, state: ExpandTaskState): void {
  assertTaskObject(task);
  registerTaskId(task.id, state.ids);

  const nextInherited = inheritedTaskState(task, inherited);

  const childTasks = task.tasks;
  if (childTasks !== undefined) {
    expandTaskGroup(task.id, childTasks, nextInherited, state);
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
  const maybeChildTasks: unknown = childTasks;
  if (!Array.isArray(maybeChildTasks) || childTasks.length === 0) {
    throw new Error("task.tasks must be a non-empty array");
  }

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

function assertTaskObject(task: unknown): asserts task is TaskDefinition {
  if (!task || typeof task !== "object") {
    throw new Error("task must be an object");
  }
  const value = task as Record<string, unknown>;
  if (typeof value.id !== "string" || value.id.trim().length === 0) {
    throw new Error("task.id must be a non-empty string");
  }
}

function registerTaskId(id: string, ids: Set<string>): void {
  if (ids.has(id)) {
    throw new Error(`duplicate task id: ${id}`);
  }
  ids.add(id);
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

function startTask<TResult>({
  task,
  execute,
  onStart,
  onComplete,
  completedIds,
  runningMutexes,
  results,
  onSettled,
  onError,
  isSettled
}: StartTaskOptions<TResult>): void {
  for (const mutex of task.mutex) {
    runningMutexes.add(mutex);
  }

  void Promise.resolve()
    .then(() => onStart(task))
    .then(() => execute(task))
    .then((result) => {
      results.push(result);
      completedIds.add(task.id);
      return onComplete(result, task);
    })
    .catch(onError)
    .finally(() => {
      if (isSettled()) {
        return;
      }
      for (const mutex of task.mutex) {
        runningMutexes.delete(mutex);
      }
      onSettled();
    });
}

function canRunTask(task: NormalizedTask, completedIds: Set<string>, runningMutexes: Set<string>): boolean {
  return task.dependsOn.every((id) => completedIds.has(id))
    && task.mutex.every((mutex) => !runningMutexes.has(mutex));
}

function describePendingTasks(pending: readonly NormalizedTask[], completedIds: Set<string>): string {
  return pending
    .map((task) => {
      const blockedBy = task.dependsOn.filter((id) => !completedIds.has(id));
      return blockedBy.length > 0 ? `${task.id} waits for ${blockedBy.join(", ")}` : task.id;
    })
    .join("; ");
}

function normalizeStringList(value: StringList, fieldName: string): string[] {
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

function normalizeTaskList(taskList: readonly TaskDefinition[]): NormalizedTask[] {
  const maybeTaskList: unknown = taskList;
  if (!Array.isArray(maybeTaskList)) {
    throw new Error("task list must be an array");
  }
  return taskList.map(normalizeTask);
}

function executeTask(task: NormalizedTask): unknown {
  if (typeof task.run !== "function") {
    throw new Error(`task ${task.id} has no run function`);
  }
  return task.run(task);
}

function noop(): void {}
