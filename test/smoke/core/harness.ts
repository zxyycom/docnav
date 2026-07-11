import { argValue, createSmokeHarness, createSmokeState, resolveBinaryPath } from "../../tools/smoke-harness.ts";
import type { SmokeCommandOptions } from "../../tools/smoke-harness.ts";
import { createCliSmokeCases } from "../../tools/cli-smoke/cases.ts";
import { expect } from "./assertions.ts";
import { logDir, logPaths, root, schemaPaths, tempRoot } from "./config.ts";

export const smokeState = createSmokeState({
  docnavBinaryPath: resolveBinaryPath(root, argValue("--bin") ?? process.env.DOCNAV_BIN)
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
  title: "Docnav Core Development Smoke",
  auditTitle: "docnav core development smoke audit",
  auditMetadata: () => [
    `temp_root: ${tempRoot}`,
    `docnav_binary: ${smokeState.docnavBinaryPath ?? "(missing)"}`
  ],
  binaryPath: () => smokeState.docnavBinaryPath ?? null,
  binaryFallback: "docnav",
  resolveCwd: (options: SmokeCommandOptions) => options.cwd ?? options.project?.root ?? root,
  resolveEnv: (options: SmokeCommandOptions) => ({
    ...process.env,
    ...(options.project?.env ?? {}),
    ...(options.env ?? {})
  }),
  safeArgPattern: /^[A-Za-z0-9_./:=@+\-\\]+$/
});

export const { runSuccessfulJsonCase } = createCliSmokeCases({
  runCli,
  validateSchema
});
