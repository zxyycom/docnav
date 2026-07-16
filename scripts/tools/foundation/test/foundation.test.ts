import { describe, expect, test } from "bun:test";

import {
  parseJsonValue,
  parsePositiveInteger,
  processFailed,
  runProcess,
  runProcessSync,
  toSlashPath
} from "../src/index.ts";

describe("script foundation", () => {
  test("parses strict positive integers", () => {
    expect(parsePositiveInteger("4", "concurrency")).toBe(4);
    expect(() => parsePositiveInteger("0", "concurrency")).toThrow("concurrency must be a positive integer");
  });

  test("parses JSON values and normalizes slash paths", () => {
    expect(parseJsonValue("{\"ok\":true}")).toEqual({ ok: true });
    expect(toSlashPath("a\\b\\c.ts")).toBe("a/b/c.ts");
  });

  test("detects failed process results", () => {
    expect(processFailed({ status: 1 })).toBe(true);
    expect(processFailed({ status: 0 })).toBe(false);
  });

  // @case AUX-WORKSPACE-PROCESS-001
  test("runs child processes with plain text output environment", async () => {
    const env = colorEnabledEnv();

    const syncResult = runProcessSync(process.execPath, childEnvProbeArgs(), { env });
    expect(syncResult.status).toBe(0);
    expect(JSON.parse(syncResult.stdout)).toEqual(plainTextEnvValues());

    const asyncResult = await runProcess({
      command: process.execPath,
      args: childEnvProbeArgs(),
      env
    });
    expect(asyncResult.status).toBe(0);
    expect(JSON.parse(asyncResult.stdout)).toEqual(plainTextEnvValues());
  });
});

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
