import fs from "node:fs";

import { fixturesDir } from "./config.mjs";
import { smokeState } from "./state.mjs";
import { assertSetup } from "./assertions.mjs";
import { compileSchemas } from "./schemas.mjs";
import { runTest } from "./runner.mjs";
import { printFailureSummary, printSuccessSummary, writeAuditLogs } from "./audit-log.mjs";

import { testReadableFindInfo, testReadableOutlineRead } from "./cases/readable.mjs";
import { testTextOutputs } from "./cases/text.mjs";
import { testManifestProbe, testProtocolOutputs, testValidInvoke } from "./cases/protocol.mjs";
import { testFixtureCorpus } from "./cases/corpus.mjs";
import { testCliArgumentCompatibilityWarnings, testCliArgumentFailures } from "./cases/cli-args.mjs";
import { testProtocolOperationErrors, testReadableOperationErrors } from "./cases/operation-errors.mjs";
import { testInvokeFailures } from "./cases/invoke-errors.mjs";

let suiteFailure = null;

try {
  smokeState.validators = compileSchemas();

  assertSetup(smokeState.binaryPath, "docnav-markdown binary path is required; pass --bin <path>");
  assertSetup(fs.existsSync(smokeState.binaryPath), `docnav-markdown binary not found: ${smokeState.binaryPath}`);
  assertSetup(fs.existsSync(fixturesDir), `fixture directory not found: ${fixturesDir}`);

  runTest("readable outline -> ref -> readable read", testReadableOutlineRead);
  runTest("readable-json output for find/info", testReadableFindInfo);
  runTest("text output for outline/read/find/info", testTextOutputs);
  runTest("protocol-json output for outline/read/find/info", testProtocolOutputs);
  runTest("manifest/probe protocol-json schemas", testManifestProbe);
  runTest("valid invoke stdin request", testValidInvoke);
  runTest("Markdown fixture corpus behavior", testFixtureCorpus);
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
