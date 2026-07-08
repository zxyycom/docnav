import { describe, expect, test } from "bun:test";

import { expandTasks, runParallelTasks } from "../src/index.ts";

describe("script parallel task runner", () => {
  test("expands task groups and preserves dependencies", () => {
    const tasks = expandTasks([
      {
        id: "group",
        tasks: [
          { id: "a", run: () => "a" },
          { id: "b", dependsOn: ["a"], run: () => "b" }
        ]
      }
    ]);

    expect(tasks.map((task) => task.id)).toEqual(["a", "b"]);
    expect(tasks[1]?.dependsOn).toEqual(["a"]);
  });

  test("runs dependency-ordered tasks", async () => {
    const order: string[] = [];
    await runParallelTasks([
      { id: "first", run: () => order.push("first") },
      { id: "second", dependsOn: ["first"], run: () => order.push("second") }
    ]);

    expect(order).toEqual(["first", "second"]);
  });
});
