import fs from "node:fs";

import { tempRoot } from "./smoke/core/config.ts";
import { assertSetup } from "./smoke/core/assertions.ts";
import {
  compileSchemas,
  printFailureSummary,
  printSuccessSummary,
  runSmokeTasks,
  smokeState,
  writeAuditLogs
} from "./smoke/core/harness.ts";
import { createCoreSmokeTasks } from "./smoke/core/profile.ts";

let suiteFailure;

try {
  smokeState.validators = compileSchemas();

  assertSetup(smokeState.docnavBinaryPath, "docnav binary path is required; pass --bin <path> or DOCNAV_BIN");
  const docnavBinaryPath = String(smokeState.docnavBinaryPath);
  assertSetup(fs.existsSync(docnavBinaryPath), `docnav binary not found: ${docnavBinaryPath}`);
  fs.mkdirSync(tempRoot, { recursive: true });

  const results = await runSmokeTasks(createCoreSmokeTasks(), {
    selector: process.env.DOCNAV_SMOKE_SELECTOR
  });
  suiteFailure = results.find((result) => !result.ok)?.error ?? null;
} catch (error) {
  suiteFailure = error;
} finally {
  writeAuditLogs();
  fs.rmSync(tempRoot, { recursive: true, force: true });
}

if (suiteFailure) {
  printFailureSummary(suiteFailure);
  process.exit(1);
}

printSuccessSummary();
