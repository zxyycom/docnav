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
import { testCliArgumentFailures } from "./cases/cli-args.mjs";
import { testConfigContextAndCompatibility } from "./cases/config-management.mjs";
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

  const tests = [
    ["real docnav + real docnav-markdown outline -> ref -> read", testRealMarkdownOutlineRefRead],
    ["document operation output matrix", testDocumentOutputMatrix],
    ["adapter selection matrix", testAdapterSelectionMatrix],
    ["CLI argument failure matrix", testCliArgumentFailures],
    ["config context and compatibility warnings", testConfigContextAndCompatibility],
    ["registry and adapter contract failure matrix", testRegistryAndContractFailures]
  ];
  for (const [label, testFn] of tests) {
    try {
      runTest(label, testFn);
    } catch (error) {
      suiteFailure ??= error;
    }
  }
} catch (error) {
  suiteFailure = error;
} finally {
  writeAuditLogs();
}

if (suiteFailure) {
  printFailureSummary(suiteFailure);
  process.exit(1);
}

fs.rmSync(tempRoot, { recursive: true, force: true });
printSuccessSummary();
