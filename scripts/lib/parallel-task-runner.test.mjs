import { describe, it } from "node:test";
import assert from "node:assert/strict";

import {
  expandTasks,
  normalizeTask,
  runParallelTasks,
  validateTaskGraph
} from "./parallel-task-runner.mjs";

describe("parallel task runner", () => {
  it("normalizes task metadata and supports task.run as the execution body", async () => {
    const task = normalizeTask({
      id: "unit",
      mutex: "shared",
      dependsOn: "setup",
      run: () => "ok"
    });

    assert.equal(task.label, "unit");
    assert.equal(task.type, "default");
    assert.deepEqual(task.mutex, ["shared"]);
    assert.deepEqual(task.dependsOn, ["setup"]);

    const results = await runParallelTasks([
      { id: "task-run-body", run: () => "done" }
    ]);

    assert.deepEqual(results, ["done"]);
  });

  it("runs independent tasks concurrently but serializes matching mutexes", async () => {
    const events = [];
    const completed = [];
    const tasks = [
      { id: "slow-independent", delayMs: 30 },
      { id: "shared-one", delayMs: 20, mutex: ["cargo-build"] },
      { id: "shared-two", delayMs: 1, mutex: ["cargo-build"] },
      { id: "fast-independent", delayMs: 1 }
    ];

    await runParallelTasks(tasks, {
      concurrency: 4,
      execute: async (task) => {
        events.push(`start:${task.id}`);
        await delay(task.delayMs);
        events.push(`end:${task.id}`);
        return task.id;
      },
      onComplete: (result) => {
        completed.push(result);
      }
    });

    assert.ok(events.indexOf("start:fast-independent") < events.indexOf("end:slow-independent"));
    assert.ok(events.indexOf("end:shared-one") < events.indexOf("start:shared-two"));
    assert.deepEqual(completed.slice(0, 2), ["fast-independent", "shared-one"]);
  });

  it("waits for topological dependencies before starting dependent tasks", async () => {
    const events = [];
    const tasks = [
      { id: "dependent", dependsOn: ["base"], delayMs: 1 },
      { id: "base", delayMs: 10 },
      { id: "independent", delayMs: 1 }
    ];

    await runParallelTasks(tasks, {
      concurrency: 3,
      execute: async (task) => {
        events.push(`start:${task.id}`);
        await delay(task.delayMs);
        events.push(`end:${task.id}`);
        return task.id;
      }
    });

    assert.ok(events.indexOf("end:base") < events.indexOf("start:dependent"));
    assert.ok(events.indexOf("start:independent") < events.indexOf("end:base"));
  });

  it("expands nested task groups with inherited metadata and group dependencies", async () => {
    const tasks = expandTasks([
      { id: "setup", run: () => "setup" },
      {
        id: "smoke",
        type: "full",
        mutex: "dev-bins",
        dependsOn: "setup",
        tasks: [
          { id: "markdown-smoke", run: () => "markdown" },
          { id: "core-smoke", mutex: "temp-root", run: () => "core" }
        ]
      },
      { id: "summary", dependsOn: "smoke", run: () => "summary" }
    ]);

    assert.deepEqual(
      tasks.map((task) => task.id),
      ["setup", "markdown-smoke", "core-smoke", "summary"]
    );
    assert.deepEqual(taskById(tasks, "markdown-smoke"), {
      id: "markdown-smoke",
      label: "markdown-smoke",
      type: "full",
      mutex: ["dev-bins"],
      dependsOn: ["setup"],
      run: taskById(tasks, "markdown-smoke").run
    });
    assert.deepEqual(taskById(tasks, "core-smoke").mutex, ["dev-bins", "temp-root"]);
    assert.deepEqual(taskById(tasks, "summary").dependsOn, ["markdown-smoke", "core-smoke"]);
  });

  it("accepts a task preparation strategy before graph validation and scheduling", async () => {
    const seen = [];

    await runParallelTasks([{ id: "group", tasks: [{ id: "leaf", run: () => "done" }] }], {
      prepareTasks: expandTasks,
      execute: (task) => {
        seen.push(task.id);
        return task.run(task);
      }
    });

    assert.deepEqual(seen, ["leaf"]);
  });

  it("rejects duplicate ids and unknown dependencies", async () => {
    assert.throws(
      () => validateTaskGraph([
        normalizeTask({ id: "same" }),
        normalizeTask({ id: "same" })
      ]),
      /duplicate task id: same/
    );

    await assert.rejects(
      () => runParallelTasks([
        { id: "dependent", dependsOn: ["missing"], run: () => "done" }
      ]),
      /task dependent depends on unknown task missing/
    );
  });
});

function delay(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

function taskById(tasks, id) {
  const task = tasks.find((candidate) => candidate.id === id);
  assert.ok(task, `expected task ${id}`);
  return task;
}
