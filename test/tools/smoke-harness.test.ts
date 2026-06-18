import { describe, it } from "node:test";
import assert from "node:assert/strict";

import { createSmokeHarness, createSmokeState, resolveSmokeConcurrency } from "./smoke-harness.ts";

describe("smoke harness task scheduling", () => {
  it("runs independent smoke tasks concurrently and keeps per-task command counts isolated", async () => {
    const state = createSmokeState();
    const events: ExternalValue[] = [];
    const harness: ExternalValue = createHarness(state, events);

    const startedAtMs = Date.now();
    const results = await harness.runSmokeTasks([
      {
        id: "slow",
        label: "slow task",
        run: async () => {
          events.push("start:slow");
          await harness.runCli("slow command", ["slow"]);
          events.push("end:slow");
        }
      },
      {
        id: "fast",
        label: "fast task",
        run: async () => {
          events.push("start:fast");
          await harness.runCli("fast one", ["fast-one"]);
          await harness.runCli("fast two", ["fast-two"]);
          events.push("end:fast");
        }
      }
    ], { concurrency: 2 });

    assert.ok(Date.now() - startedAtMs < 80);
    assert.ok(events.indexOf("start:fast") < events.indexOf("end:slow"));
    assert.deepEqual(results.map((result: ExternalValue) => result.ok), [true, true]);
    assert.deepEqual(
      state.testResults.map((result: ExternalValue) => [result.label, result.commandCount]),
      [["slow task", 1], ["fast task", 2]]
    );
    assert.deepEqual(
      state.commandRecords.map((record: ExternalValue) => record.name).sort(),
      ["fast one", "fast two", "slow command"]
    );
  });

  it("records failed task results without stopping other independent tasks", async () => {
    const state = createSmokeState();
    const harness = createHarness(state, []);

    const results = await harness.runSmokeTasks([
      {
        id: "fails",
        label: "failing task",
        run: async () => {
          await harness.runCli("failing command", ["fail"]);
          throw new Error("expected failure");
        }
      },
      {
        id: "passes",
        label: "passing task",
        run: async () => {
          await harness.runCli("passing command", ["pass"]);
        }
      }
    ], { concurrency: 2 });

    assert.deepEqual(results.map((result: ExternalValue) => result.ok).sort(), [false, true]);
    assert.equal(state.testResults.length, 2);
    const failingResult = state.testResults.find((result) => result.label === "failing task");
    assert.equal(failingResult?.error?.message, "expected failure");
    assert.deepEqual(
      state.commandRecords.map((record: ExternalValue) => record.name).sort(),
      ["failing command", "passing command"]
    );
  });

  it("runs nested case tasks but records only the parent smoke report", async () => {
    const state = createSmokeState();
    const harness = createHarness(state, []);

    const results = await harness.runSmokeTasks([
      {
        id: "matrix",
        label: "case matrix",
        tasks: [
          {
            id: "case-one",
            run: async () => {
              await harness.runCli("case one command", ["case-one"]);
            }
          },
          {
            id: "case-two",
            run: async () => {
              await harness.runCli("case two command", ["case-two"]);
            }
          }
        ]
      }
    ], { concurrency: 2 });

    assert.equal(results.length, 1);
    assert.deepEqual(
      state.testResults.map((result: ExternalValue) => [result.label, result.commandCount]),
      [["case matrix", 2]]
    );
  });

  it("validates smoke concurrency values", () => {
    assert.equal(resolveSmokeConcurrency(undefined), undefined);
    assert.equal(resolveSmokeConcurrency(""), undefined);
    assert.equal(resolveSmokeConcurrency("2"), 2);
    assert.throws(() => resolveSmokeConcurrency("0"), /positive integer/);
    assert.throws(() => resolveSmokeConcurrency("abc"), /positive integer/);
  });
});

function createHarness(state: ExternalValue, events: ExternalValue) {
  return createSmokeHarness({
    state,
    root: process.cwd(),
    logDir: process.cwd(),
    logPaths: [],
    schemaPaths: {},
    expect,
    title: "test smoke",
    auditTitle: "test smoke audit",
    auditMetadata: () => [],
    binaryPath: () => process.execPath,
    binaryFallback: "node",
    resolveCwd: () => process.cwd(),
    safeArgPattern: /^[A-Za-z0-9_./:=@+-]+$/,
    runProcess: async (_executable: ExternalValue, args: ExternalValue) => {
      await delay(args[0] === "slow" ? 40 : 1);
      events.push(`command:${args[0]}`);
      return {
        exitCode: 0,
        signal: null,
        error: null,
        stdout: "",
        stderr: ""
      };
    }
  });
}

function expect(record: ExternalValue, condition: ExternalValue, summary: ExternalValue) {
  const ok = Boolean(condition);
  record.assertions.push({ ok, summary });
  if (!ok) {
    throw new Error(`${record.name}: ${summary}`);
  }
}

function delay(ms: ExternalValue) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}
