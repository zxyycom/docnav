import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { createSmokeHarness, createSmokeState } from "./smoke-harness.ts";
import type { CommandRecord, SmokeState } from "./smoke-harness.ts";

describe("smoke harness task scheduling", () => {
  it("runs independent smoke tasks concurrently and keeps per-task command counts isolated", async () => {
    const state = createSmokeState();
    const slowStarted = Promise.withResolvers<void>();
    const releaseSlow = Promise.withResolvers<void>();
    const fastFinished = Promise.withResolvers<void>();
    let slowFinished = false;
    const harness = createHarness(state, async (_executable, args) => {
      if (args[0] === "slow") {
        slowStarted.resolve();
        await releaseSlow.promise;
        slowFinished = true;
      }
      return successfulProcessResult();
    });

    const running = harness.runSmokeTasks([
      {
        id: "slow",
        label: "slow task",
        run: async () => {
          await harness.runCli("slow command", ["slow"]);
        }
      },
      {
        id: "fast",
        label: "fast task",
        run: async () => {
          await harness.runCli("fast one", ["fast-one"]);
          await harness.runCli("fast two", ["fast-two"]);
          fastFinished.resolve();
        }
      }
    ], { concurrency: 2 });

    await Promise.all([slowStarted.promise, fastFinished.promise]);
    assert.equal(slowFinished, false);
    releaseSlow.resolve();

    const results = await running;
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
    const harness = createHarness(state);

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
    const harness = createHarness(state);

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

  it("selects one smoke leaf by its stable id and preserves the parent report", async () => {
    const state = createSmokeState();
    const harness = createHarness(state);

    const results = await harness.runSmokeTasks([
      {
        id: "matrix",
        label: "case matrix",
        tasks: [
          {
            id: "case-one",
            label: "case one",
            run: async () => {
              await harness.runCli("case one command", ["case-one"]);
            }
          },
          {
            id: "case-two",
            label: "case two",
            run: async () => {
              await harness.runCli("case two command", ["case-two"]);
            }
          }
        ]
      }
    ], {
      selector: "case-two"
    });

    assert.deepEqual(
      state.commandRecords.map(({ name }) => name),
      ["case two command"]
    );
    assert.deepEqual(
      results.map(({ id, label, commandCount }) => ({
        id,
        label,
        commandCount
      })),
      [
        {
          id: "matrix",
          label: "case matrix",
          commandCount: 1
        }
      ]
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
    assert.equal(record.executable, process.execPath);
    assert.match(
      harness.formatCommandRecord(record).join("\n"),
      new RegExp(`^\\$ ${escapeRegex(process.execPath)} `, "m")
    );
    assert.equal(state.commandRecords[0], record);
  });

  it("runs default runner commands with plain text output environment", async () => {
    const state = createSmokeState();
    const harness = createSpawnHarness(state);

    const record = await harness.runCli("plain env", childEnvProbeArgs(), {
      env: colorEnabledEnv()
    });

    assert.equal(record.exitCode, 0);
    assert.deepEqual(JSON.parse(record.stdout), plainTextEnvValues());
  });

  it("creates and cleans only the owned core smoke run directory after task failure", { timeout: 10_000 }, () => {
    const tempBase = path.join(
      os.tmpdir(),
      `docnav-core-smoke-cleanup-${process.pid}-${Date.now()}`
    );
    const markerPath = path.join(tempBase, "caller-owned-marker");
    const probePath = path.join(tempBase, "fake-docnav-cwds.txt");
    fs.mkdirSync(tempBase, { recursive: true });
    fs.writeFileSync(markerPath, "cleanup fixture");
    fs.writeFileSync(probePath, "");
    const fakeBinary = createFailingDocnavBinary(tempBase, probePath);
    const callerOwnedEntries = fs.readdirSync(tempBase).sort();

    try {
      const result = spawnSync(process.execPath, ["test/docnav-core-smoke.ts"], {
        cwd: process.cwd(),
        encoding: "utf8",
        env: {
          ...process.env,
          DOCNAV_BIN: fakeBinary,
          DOCNAV_CORE_SMOKE_TEMP_ROOT: tempBase,
          DOCNAV_SMOKE_PROBE_PATH: probePath
        },
        timeout: 10_000
      });

      assert.notEqual(result.status, 0, "fixture should exercise the failing smoke path");
      const observedProjectCwds = fs.readFileSync(probePath, "utf8").trim().split(/\r?\n/u).filter(Boolean);
      assert.ok(observedProjectCwds.length > 0, "fake binary should run after smoke projects are created");
      for (const projectCwd of observedProjectCwds) {
        const relative = path.relative(tempBase, projectCwd);
        assert.ok(relative && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative));
        assert.equal(fs.existsSync(projectCwd), false, "owned smoke project should be removed after failure");
      }
      assert.equal(fs.readFileSync(markerPath, "utf8"), "cleanup fixture");
      assert.deepEqual(
        fs.readdirSync(tempBase).sort(),
        callerOwnedEntries,
        "smoke cleanup should remove its run directory and preserve the caller-owned base"
      );
    } finally {
      fs.rmSync(tempBase, { recursive: true, force: true });
    }
  });
});

type HarnessRunProcess = NonNullable<
  Parameters<typeof createSmokeHarness>[0]["runProcess"]
>;

function createHarness(state: SmokeState, runProcess?: HarnessRunProcess) {
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
    runProcess: runProcess ?? (async () => successfulProcessResult())
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

function createFailingDocnavBinary(tempBase: string, probePath: string): string {
  const scriptPath = path.join(tempBase, "fake-docnav.cjs");
  fs.writeFileSync(
    scriptPath,
    [
      `#!${process.execPath}`,
      'const fs = require("node:fs");',
      `fs.appendFileSync(${JSON.stringify(probePath)}, process.cwd() + "\\n");`,
      'process.stderr.write("intentional docnav smoke failure\\n");',
      "process.exit(7);"
    ].join("\n"),
    "utf8"
  );

  if (process.platform !== "win32") {
    fs.chmodSync(scriptPath, 0o755);
    return scriptPath;
  }

  const commandPath = path.join(tempBase, "fake-docnav.cmd");
  fs.writeFileSync(
    commandPath,
    `@echo off\r\n"${process.execPath}" "${scriptPath}" %*\r\n`,
    "utf8"
  );
  return commandPath;
}

function expect(record: CommandRecord, condition: unknown, summary: string) {
  const ok = Boolean(condition);
  record.assertions.push({ ok, summary });
  if (!ok) {
    throw new Error(`${record.name}: ${summary}`);
  }
}

function successfulProcessResult() {
  return {
    exitCode: 0,
    signal: null,
    error: null,
    stdout: "",
    stderr: ""
  };
}

function escapeRegex(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function childEnvProbeArgs(): string[] {
  const keys = Object.keys(plainTextEnvValues());
  return [
    "-e",
    [
      `const keys = ${JSON.stringify(keys)};`,
      "process.stdout.write(JSON.stringify(Object.fromEntries(keys.map((key) => [key, process.env[key] ?? null]))));"
    ].join(" ")
  ];
}

function colorEnabledEnv(): NodeJS.ProcessEnv {
  return {
    ...process.env,
    CARGO_TERM_COLOR: "always",
    CLICOLOR: "1",
    CLICOLOR_FORCE: "1",
    FORCE_COLOR: "1",
    NO_COLOR: "0",
    PNPM_CONFIG_COLOR: "true",
    PY_COLORS: "1",
    TERM: "xterm-256color",
    UV_NO_COLOR: "0",
    npm_config_color: "true"
  };
}

function plainTextEnvValues() {
  return {
    CARGO_TERM_COLOR: "never",
    CLICOLOR: "0",
    CLICOLOR_FORCE: "0",
    FORCE_COLOR: "0",
    NO_COLOR: "1",
    PNPM_CONFIG_COLOR: "false",
    PY_COLORS: "0",
    TERM: "dumb",
    UV_NO_COLOR: "1",
    npm_config_color: "false"
  };
}
