import { argValue, createSmokeHarness, createSmokeState, resolveBinaryPath } from "../../tools/smoke-harness.ts";
import type { SmokeCommandOptions } from "../../tools/smoke-harness.ts";
import { createCliSmokeCases } from "../../tools/cli-smoke/cases.ts";
import { expect } from "./assertions.ts";
import { logDir, logPaths, root, schemaPaths, tempRoot } from "./config.ts";

export const smokeState = createSmokeState({
  binaryPath: resolveBinaryPath(root, argValue("--bin") ?? process.env.DOCNAV_MARKDOWN_BIN),
  normalRef: null
});

export const {
  compileSchemas,
  formatAssertions,
  formatCommandRecord,
  printFailureSummary,
  printSuccessSummary,
  runCli,
  runSmokeTasks,
  runTest,
  validateSchema,
  writeAuditLogs
} = createSmokeHarness({
  state: smokeState,
  root,
  logDir,
  logPaths,
  schemaPaths,
  expect,
  title: "Docnav Markdown Development Smoke",
  auditTitle: "docnav-markdown development smoke audit",
  auditMetadata: () => [
    `temp_root: ${tempRoot}`,
    `binary: ${smokeState.binaryPath ?? "(missing)"}`
  ],
  binaryPath: () => smokeState.binaryPath ?? null,
  binaryFallback: "docnav-markdown",
  resolveCwd: (options: SmokeCommandOptions) => options.cwd ?? root,
  resolveEnv: (options: SmokeCommandOptions) => ({
    ...process.env,
    ...(options.env ?? {})
  }),
  safeArgPattern: /^[A-Za-z0-9_./:=@+\-\\]+$/
});

export const { runProtocolResponseCase, runSuccessfulJsonCase } = createCliSmokeCases({
  runCli,
  validateSchema
});
