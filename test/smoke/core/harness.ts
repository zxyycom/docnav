import { argValue, createSmokeHarness, createSmokeState, resolveBinaryPath } from "../../tools/smoke-harness.ts";
import { createCliSmokeCases } from "../../tools/cli-smoke/cases.ts";
import { expect } from "./assertions.ts";
import { logDir, logPaths, root, schemaPaths, tempRoot } from "./config.ts";

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
  binaryPath: () => smokeState.docnavBinaryPath ?? null,
  binaryFallback: "docnav",
  resolveCwd: (options: ExternalValue) => options.cwd ?? options.project?.root ?? root,
  resolveEnv: (options: ExternalValue) => ({
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
