import type { NormalizedTask } from "./definition.ts";

export interface TaskSchedulerOptions<TResult> {
  pending: NormalizedTask[];
  concurrency: number;
  execute: (task: NormalizedTask) => TResult | Promise<TResult>;
  onStart: (task: NormalizedTask) => unknown | Promise<unknown>;
  onComplete: (result: TResult, task: NormalizedTask) => unknown | Promise<unknown>;
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

interface TaskScheduler<TResult> extends TaskSchedulerOptions<TResult> {
  completedIds: Set<string>;
  runningMutexes: Set<string>;
  results: TResult[];
  activeCount: number;
  settled: boolean;
}

export async function runTaskScheduler<TResult>(options: TaskSchedulerOptions<TResult>): Promise<TResult[]> {
  const scheduler = createTaskScheduler(options);

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

      failIfBlocked(scheduler, fail);
      finishIfDone();
    };

    schedule();
  });

  return scheduler.results;
}

function createTaskScheduler<TResult>(options: TaskSchedulerOptions<TResult>): TaskScheduler<TResult> {
  return {
    ...options,
    completedIds: new Set<string>(),
    runningMutexes: new Set<string>(),
    results: [],
    activeCount: 0,
    settled: false
  };
}

function scheduleReadyTasks<TResult>(
  scheduler: TaskScheduler<TResult>,
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

function failIfBlocked<TResult>(scheduler: TaskScheduler<TResult>, fail: (error: unknown) => void): void {
  if (scheduler.activeCount === 0 && scheduler.pending.length > 0) {
    fail(new Error(`unable to schedule tasks; unresolved dependencies or cycle: ${describePendingTasks(scheduler.pending, scheduler.completedIds)}`));
  }
}

function completeSchedulerIfDone<TResult>(scheduler: TaskScheduler<TResult>): boolean {
  if (scheduler.settled || scheduler.pending.length > 0 || scheduler.activeCount > 0) {
    return false;
  }
  scheduler.settled = true;
  return true;
}

function failScheduler<TResult>(scheduler: TaskScheduler<TResult>): boolean {
  if (scheduler.settled) {
    return false;
  }
  scheduler.settled = true;
  return true;
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
