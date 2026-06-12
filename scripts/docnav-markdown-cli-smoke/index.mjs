import fs from "node:fs";

import { fixturesDir } from "./config.mjs";
import { assertSetup } from "./assertions.mjs";
import {
  compileSchemas,
  printFailureSummary,
  printSuccessSummary,
  runTest,
  smokeState,
  writeAuditLogs
} from "./harness.mjs";

import { testDocumentOutputMatrix } from "./cases/outputs.mjs";
import { testManifestProbe, testValidInvoke } from "./cases/machine-commands.mjs";
import { testProcessBoundaryCorpus } from "./cases/corpus.mjs";
import { testCliArgumentCompatibilityWarnings, testCliArgumentFailures } from "./cases/cli-args.mjs";
import { testProtocolOperationErrors, testReadableOperationErrors } from "./cases/operation-errors.mjs";
import { testInvokeFailures } from "./cases/invoke-errors.mjs";

let suiteFailure = null;

try {
  smokeState.validators = compileSchemas();

  assertSetup(smokeState.binaryPath, "docnav-markdown binary path is required; pass --bin <path>");
  assertSetup(fs.existsSync(smokeState.binaryPath), `docnav-markdown binary not found: ${smokeState.binaryPath}`);
  assertSetup(fs.existsSync(fixturesDir), `fixture directory not found: ${fixturesDir}`);

  runTest("document operation output matrix", testDocumentOutputMatrix);
  runTest("manifest/probe protocol-json schemas", testManifestProbe);
  runTest("valid invoke stdin request", testValidInvoke);
  runTest("Markdown process boundary corpus", testProcessBoundaryCorpus);
  runTest("CLI argument validation matrix", testCliArgumentFailures);
  runTest("CLI argument compatibility warning matrix", testCliArgumentCompatibilityWarnings);
  runTest("readable operation error matrix", testReadableOperationErrors);
  runTest("protocol-json operation error matrix", testProtocolOperationErrors);
  runTest("invoke malformed/schema error matrix", testInvokeFailures);
} catch (error) {
  suiteFailure = error;
} finally {
  writeAuditLogs();
}

if (suiteFailure) {
  printFailureSummary(suiteFailure);
  process.exit(1);
}

printSuccessSummary();
