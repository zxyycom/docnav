import { argValue, createSmokeHarness, createSmokeState, resolveBinaryPath } from "../smoke-harness.mjs";
import { createCliSmokeCases } from "../cli-smoke/cases.mjs";
import { expect } from "./assertions.mjs";
import { logDir, logPaths, root, schemaPaths, tempRoot } from "./config.mjs";

export const smokeState = createSmokeState({
  docnavBinaryPath: resolveBinaryPath(root, argValue("--bin") ?? process.env.DOCNAV_BIN),
  markdownBinaryPath: resolveBinaryPath(root, process.env.DOCNAV_MARKDOWN_BIN)
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
    `docnav_binary: ${smokeState.docnavBinaryPath ?? "(missing)"}`,
    `docnav_markdown_binary: ${smokeState.markdownBinaryPath ?? "(missing)"}`
  ],
  binaryPath: () => smokeState.docnavBinaryPath,
  binaryFallback: "docnav",
  resolveCwd: (options) => options.cwd ?? options.project?.root ?? root,
  resolveEnv: (options) => ({
    ...process.env,
    ...(options.project?.env ?? {}),
    ...(options.env ?? {})
  }),
  safeArgPattern: /^[A-Za-z0-9_./:=@+\-\\]+$/
});

export const { runProtocolResponseCase, runSuccessfulJsonCase } = createCliSmokeCases({
  runCli,
  validateSchema
});
