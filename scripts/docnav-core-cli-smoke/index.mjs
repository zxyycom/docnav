import fs from "node:fs";

import { tempRoot } from "./config.mjs";
import { smokeState } from "./state.mjs";
import { assertSetup } from "./assertions.mjs";
import { compileSchemas } from "./schemas.mjs";
import { runTest } from "./runner.mjs";
import { printFailureSummary, printSuccessSummary, writeAuditLogs } from "./audit-log.mjs";

import { testRealMarkdownOutlineRefRead } from "./cases/real-markdown.mjs";
import { testDocumentOutputMatrix } from "./cases/outputs.mjs";
import { testAdapterSelectionMatrix } from "./cases/adapter-selection.mjs";
import { testConfigManagementAndCompatibility } from "./cases/config-management.mjs";
import { testRegistryAndContractFailures } from "./cases/failures.mjs";

let suiteFailure = null;

try {
  smokeState.validators = compileSchemas();

  assertSetup(smokeState.docnavBinaryPath, "docnav binary path is required; pass --bin <path> or DOCNAV_BIN");
  assertSetup(fs.existsSync(smokeState.docnavBinaryPath), `docnav binary not found: ${smokeState.docnavBinaryPath}`);
  assertSetup(smokeState.markdownBinaryPath, "docnav-markdown binary path is required; set DOCNAV_MARKDOWN_BIN");
  assertSetup(
    fs.existsSync(smokeState.markdownBinaryPath),
    `docnav-markdown binary not found: ${smokeState.markdownBinaryPath}`
  );
  fs.mkdirSync(tempRoot, { recursive: true });

  runTest("real docnav + real docnav-markdown outline -> ref -> read", testRealMarkdownOutlineRefRead);
  runTest("document operation output matrix", testDocumentOutputMatrix);
  runTest("adapter selection matrix", testAdapterSelectionMatrix);
  runTest("config, management, and compatibility warnings", testConfigManagementAndCompatibility);
  runTest("registry and adapter contract failure matrix", testRegistryAndContractFailures);
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

