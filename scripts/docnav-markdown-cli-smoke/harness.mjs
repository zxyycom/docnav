import { argValue, createSmokeHarness, createSmokeState, resolveBinaryPath } from "../smoke-harness.mjs";
import { createCliSmokeCases } from "../cli-smoke/cases.mjs";
import { expect } from "./assertions.mjs";
import { logDir, logPaths, root, schemaPaths } from "./config.mjs";

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
  binaryPath: () => smokeState.binaryPath,
  binaryFallback: "docnav-markdown"
});

export const { runProtocolResponseCase, runSuccessfulJsonCase } = createCliSmokeCases({
  runCli,
  validateSchema
});
