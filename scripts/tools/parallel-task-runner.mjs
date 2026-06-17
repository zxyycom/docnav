export async function runParallelTasks(taskList, options = {}) {
  const prepareTasks = options.prepareTasks ?? normalizeTaskList;
  const execute = options.execute ?? executeTask;
  const onStart = options.onStart ?? noop;
  const onComplete = options.onComplete ?? noop;
  const pending = prepareTasks(taskList).map(normalizeTask);
  const concurrency = resolveConcurrency(options.concurrency, pending.length);
  validateTaskGraph(pending);

  const completedIds = new Set();
  const runningMutexes = new Set();
  const results = [];
  let activeCount = 0;
  let settled = false;

  await new Promise((resolve, reject) => {
    const finishIfDone = () => {
      if (!settled && pending.length === 0 && activeCount === 0) {
        settled = true;
        resolve();
      }
    };

    const fail = (error) => {
      if (!settled) {
        settled = true;
        reject(error);
      }
    };

    const schedule = () => {
      while (activeCount < concurrency) {
        const nextIndex = pending.findIndex((task) => canRunTask(task, completedIds, runningMutexes));
        if (nextIndex === -1) {
          break;
        }

        const [task] = pending.splice(nextIndex, 1);
        startTask({
          task,
          execute,
          onStart,
          onComplete,
          completedIds,
          runningMutexes,
          results,
          onSettled: () => {
            activeCount -= 1;
            schedule();
            finishIfDone();
          },
          onError: fail,
          isSettled: () => settled
        });
        activeCount += 1;
      }

      if (activeCount === 0 && pending.length > 0) {
        fail(new Error(`unable to schedule tasks; unresolved dependencies or cycle: ${describePendingTasks(pending, completedIds)}`));
        return;
      }
      finishIfDone();
    };

    schedule();
  });

  return results;
}

function resolveConcurrency(value, taskCount) {
  if (value === undefined || value === null) {
    return taskCount;
  }

  const parsed = Number.parseInt(String(value), 10);
  if (!Number.isFinite(parsed) || parsed < 1 || String(parsed) !== String(value)) {
    throw new Error(`task concurrency must be a positive integer: ${value}`);
  }
  return parsed;
}

export function expandTasks(taskList) {
  if (!Array.isArray(taskList)) {
    throw new Error("task list must be an array");
  }

  const ids = new Set();
  const groupLeafIds = new Map();
  const leafTasks = [];

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

export function normalizeTask(task) {
  if (!task || typeof task !== "object") {
    throw new Error("task must be an object");
  }
  if (typeof task.id !== "string" || task.id.trim().length === 0) {
    throw new Error("task.id must be a non-empty string");
  }

  return {
    label: task.id,
    type: "default",
    ...task,
    mutex: normalizeStringList(task.mutex, "mutex"),
    dependsOn: normalizeStringList(task.dependsOn, "dependsOn")
  };
}

export function validateTaskGraph(taskList) {
  const ids = new Set();
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

function expandTask(task, inherited, state) {
  assertTaskObject(task);
  registerTaskId(task.id, state.ids);

  const taskMutex = normalizeStringList(task.mutex, "mutex");
  const taskDependsOn = normalizeStringList(task.dependsOn, "dependsOn");
  const taskEnv = mergeEnv(inherited.env, task.env);
  const taskEnvFile = task.envFile ?? inherited.envFile;
  const taskType = task.type ?? inherited.type;
  const nextInherited = {
    type: taskType,
    mutex: [...inherited.mutex, ...taskMutex],
    dependsOn: [...inherited.dependsOn, ...taskDependsOn],
    env: taskEnv,
    envFile: taskEnvFile
  };

  if (task.tasks !== undefined) {
    if (!Array.isArray(task.tasks) || task.tasks.length === 0) {
      throw new Error("task.tasks must be a non-empty array");
    }

    const startIndex = state.leafTasks.length;
    for (const child of task.tasks) {
      expandTask(child, nextInherited, state);
    }
    state.groupLeafIds.set(
      task.id,
      state.leafTasks.slice(startIndex).map((leaf) => leaf.id)
    );
    return;
  }

  const {
    dependsOn: _dependsOn,
    env: _env,
    envFile: _envFile,
    mutex: _mutex,
    tasks: _tasks,
    type: _type,
    ...rest
  } = task;
  const leaf = {
    type: taskType,
    ...rest,
    mutex: nextInherited.mutex,
    dependsOn: nextInherited.dependsOn
  };
  if (taskEnv !== undefined) {
    leaf.env = taskEnv;
  }
  if (taskEnvFile !== undefined) {
    leaf.envFile = taskEnvFile;
  }
  state.leafTasks.push(normalizeTask(leaf));
}

function assertTaskObject(task) {
  if (!task || typeof task !== "object") {
    throw new Error("task must be an object");
  }
  if (typeof task.id !== "string" || task.id.trim().length === 0) {
    throw new Error("task.id must be a non-empty string");
  }
}

function registerTaskId(id, ids) {
  if (ids.has(id)) {
    throw new Error(`duplicate task id: ${id}`);
  }
  ids.add(id);
}

function mergeEnv(parentEnv, taskEnv) {
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

function resolveGroupDependencies(dependsOn, groupLeafIds, taskId) {
  const resolved = [];
  const seen = new Set();
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

function startTask({
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
}) {
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

function canRunTask(task, completedIds, runningMutexes) {
  return task.dependsOn.every((id) => completedIds.has(id))
    && task.mutex.every((mutex) => !runningMutexes.has(mutex));
}

function describePendingTasks(pending, completedIds) {
  return pending
    .map((task) => {
      const blockedBy = task.dependsOn.filter((id) => !completedIds.has(id));
      return blockedBy.length > 0 ? `${task.id} waits for ${blockedBy.join(", ")}` : task.id;
    })
    .join("; ");
}

function normalizeStringList(value, fieldName) {
  if (value === undefined) {
    return [];
  }
  if (typeof value === "string") {
    return [value];
  }
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string" || item.length === 0)) {
    throw new Error(`task.${fieldName} must be a string or string array`);
  }
  return [...value];
}

function normalizeTaskList(taskList) {
  if (!Array.isArray(taskList)) {
    throw new Error("task list must be an array");
  }
  return taskList.map(normalizeTask);
}

function executeTask(task) {
  if (typeof task.run !== "function") {
    throw new Error(`task ${task.id} has no run function`);
  }
  return task.run(task);
}

function noop() {}
