import { argValue, createSmokeHarness, createSmokeState, resolveBinaryPath } from "../../tools/smoke-harness.ts";
import { createCliSmokeCases } from "../../tools/cli-smoke/cases.ts";
import { expect } from "./assertions.ts";
import { logDir, logPaths, root, schemaPaths } from "./config.ts";

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
  auditMetadata: () => [`binary: ${smokeState.binaryPath ?? "(missing)"}`],
  binaryPath: () => smokeState.binaryPath ?? null,
  binaryFallback: "docnav-markdown"
});

export const { runProtocolResponseCase, runSuccessfulJsonCase } = createCliSmokeCases({
  runCli,
  validateSchema
});
