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

  it("uses DOCNAV_SMOKE_CONCURRENCY at the smoke scheduling boundary", async () => {
    const state = createSmokeState();
    const events: string[] = [];
    const harness = createHarness(state, events);
    const previous = process.env.DOCNAV_SMOKE_CONCURRENCY;
    process.env.DOCNAV_SMOKE_CONCURRENCY = "1";

    try {
      await harness.runSmokeTasks([
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
            await harness.runCli("fast command", ["fast"]);
            events.push("end:fast");
          }
        }
      ]);
    } finally {
      restoreEnv("DOCNAV_SMOKE_CONCURRENCY", previous);
    }

    assert.ok(events.indexOf("end:slow") < events.indexOf("start:fast"));
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

  it("runs default runner commands with plain text output environment", async () => {
    const state = createSmokeState();
    const harness = createSpawnHarness(state);

    const record = await harness.runCli("plain env", childEnvProbeArgs(), {
      env: colorEnabledEnv()
    });

    assert.equal(record.exitCode, 0);
    assert.deepEqual(JSON.parse(record.stdout), plainTextEnvValues());
  });

  it("validates smoke concurrency values", () => {
    const previous = process.env.DOCNAV_SMOKE_CONCURRENCY;
    process.env.DOCNAV_SMOKE_CONCURRENCY = "4";

    try {
      assert.equal(resolveSmokeConcurrency(undefined), undefined);
      assert.equal(resolveSmokeConcurrency(""), undefined);
      assert.equal(resolveSmokeConcurrency("2"), 2);
      assert.throws(() => resolveSmokeConcurrency("0"), /positive integer/);
      assert.throws(() => resolveSmokeConcurrency("abc"), /positive integer/);
    } finally {
      restoreEnv("DOCNAV_SMOKE_CONCURRENCY", previous);
    }
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

function delay(ms: number) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

function restoreEnv(key: string, value: string | undefined) {
  if (value === undefined) {
    delete process.env[key];
    return;
  }
  process.env[key] = value;
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
