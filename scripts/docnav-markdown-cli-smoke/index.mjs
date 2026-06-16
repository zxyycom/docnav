import fs from "node:fs";

import { fixturesDir } from "./config.mjs";
import { assertSetup } from "./assertions.mjs";
import {
  compileSchemas,
  printFailureSummary,
  printSuccessSummary,
  runSmokeTasks,
  smokeState,
  writeAuditLogs
} from "./harness.mjs";

import { createDocumentOutputMatrixTasks } from "./cases/outputs.mjs";
import { createManifestProbeTasks, createValidInvokeTasks } from "./cases/machine-commands.mjs";
import { createProcessBoundaryCorpusTasks } from "./cases/corpus.mjs";
import { createCliArgumentCompatibilityWarningTasks, createCliArgumentFailureTasks } from "./cases/cli-args.mjs";
import { createProtocolOperationErrorTasks, createReadableOperationErrorTasks } from "./cases/operation-errors.mjs";
import { createInvokeFailureTasks } from "./cases/invoke-errors.mjs";

let suiteFailure = null;

try {
  smokeState.validators = compileSchemas();

  assertSetup(smokeState.binaryPath, "docnav-markdown binary path is required; pass --bin <path>");
  assertSetup(fs.existsSync(smokeState.binaryPath), `docnav-markdown binary not found: ${smokeState.binaryPath}`);
  assertSetup(fs.existsSync(fixturesDir), `fixture directory not found: ${fixturesDir}`);

  const results = await runSmokeTasks([
    { id: "document-output-matrix", label: "document operation output matrix", tasks: createDocumentOutputMatrixTasks() },
    { id: "manifest-probe", label: "manifest/probe protocol-json schemas", tasks: createManifestProbeTasks() },
    { id: "valid-invoke", label: "valid invoke stdin request", tasks: createValidInvokeTasks() },
    { id: "process-boundary-corpus", label: "Markdown process boundary corpus", tasks: createProcessBoundaryCorpusTasks() },
    { id: "cli-argument-failures", label: "CLI argument validation matrix", tasks: createCliArgumentFailureTasks() },
    {
      id: "cli-argument-compatibility",
      label: "CLI argument compatibility warning matrix",
      tasks: createCliArgumentCompatibilityWarningTasks()
    },
    { id: "readable-operation-errors", label: "readable operation error matrix", tasks: createReadableOperationErrorTasks() },
    { id: "protocol-operation-errors", label: "protocol-json operation error matrix", tasks: createProtocolOperationErrorTasks() },
    { id: "invoke-failures", label: "invoke malformed/schema error matrix", tasks: createInvokeFailureTasks() }
  ]);
  suiteFailure = results.find((result) => !result.ok)?.error ?? null;
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
