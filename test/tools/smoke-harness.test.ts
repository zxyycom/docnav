import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { createSmokeHarness, createSmokeState, resolveSmokeConcurrency } from "./smoke-harness.ts";
import type { CommandRecord, SmokeState } from "./smoke-harness.ts";

// @case AUX-SMOKE-HARNESS-001
describe("smoke harness task scheduling", () => {
  it("runs independent smoke tasks concurrently and keeps per-task command counts isolated", async () => {
    const state = createSmokeState();
    const events: string[] = [];
    const harness = createHarness(state, events);

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

    assert.ok(events.indexOf("start:fast") < events.indexOf("end:slow"));
    assert.ok(events.indexOf("start:fast") < events.indexOf("command:slow"));
    assert.deepEqual(results.map((result) => result.ok), [true, true]);
    assert.deepEqual(
      state.testResults.map((result) => [result.label, result.commandCount]),
      [["slow task", 1], ["fast task", 2]]
    );
    assert.deepEqual(
      state.commandRecords.map((record) => record.name).sort(),
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

    assert.deepEqual(results.map((result) => result.ok).sort(), [false, true]);
    assert.equal(state.testResults.length, 2);
    const failingResult = state.testResults.find((result) => result.label === "failing task");
    assert.equal(failingResult?.error?.message, "expected failure");
    assert.deepEqual(
      state.commandRecords.map((record) => record.name).sort(),
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
      state.testResults.map((result) => [result.label, result.commandCount]),
      [["case matrix", 2]]
    );
  });

  it("records default runner stdout and stderr on command records", async () => {
    const state = createSmokeState();
    const harness = createSpawnHarness(state);

    const record = await harness.runCli("node output", [
      "-e",
      "process.stdout.write('out'); process.stderr.write('err');"
    ]);

    assert.equal(record.exitCode, 0);
    assert.equal(record.stdout, "out");
    assert.equal(record.stderr, "err");
    assert.equal(state.commandRecords[0], record);
  });

  it("validates smoke concurrency values", () => {
    assert.equal(resolveSmokeConcurrency(undefined), undefined);
    assert.equal(resolveSmokeConcurrency(""), undefined);
    assert.equal(resolveSmokeConcurrency("2"), 2);
    assert.throws(() => resolveSmokeConcurrency("0"), /positive integer/);
    assert.throws(() => resolveSmokeConcurrency("abc"), /positive integer/);
  });

  it("cleans the core smoke temp root when the suite exits after failure", { timeout: 10_000 }, () => {
    const tempRoot = path.join(
      os.tmpdir(),
      `docnav-core-smoke-cleanup-${process.pid}-${Date.now()}`
    );
    fs.mkdirSync(tempRoot, { recursive: true });
    fs.writeFileSync(path.join(tempRoot, "marker"), "cleanup fixture");

    const result = spawnSync(process.execPath, ["test/docnav-core-smoke.ts"], {
      cwd: process.cwd(),
      encoding: "utf8",
      env: {
        ...process.env,
        DOCNAV_BIN: path.join(tempRoot, "missing-docnav"),
        DOCNAV_CORE_SMOKE_TEMP_ROOT: tempRoot
      },
      timeout: 10_000
    });

    const tempRootExists = fs.existsSync(tempRoot);
    fs.rmSync(tempRoot, { recursive: true, force: true });

    assert.notEqual(result.status, 0, "fixture should exercise the failing smoke path");
    assert.match(result.stderr, /docnav binary not found:/);
    assert.equal(
      tempRootExists,
      false,
      `temp root should be cleaned after failure\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`
    );
  });
});

function createHarness(state: SmokeState, events: string[]) {
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
    runProcess: async (_executable: string, args: string[]) => {
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

function createSpawnHarness(state: SmokeState) {
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
    safeArgPattern: /^[A-Za-z0-9_./:=@+-]+$/
  });
}

function expect(record: CommandRecord, condition: unknown, summary: string) {
  const ok = Boolean(condition);
  record.assertions.push({ ok, summary });
  if (!ok) {
    throw new Error(`${record.name}: ${summary}`);
  }
}

function delay(ms: number) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}
