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

import { createDocumentLinkTasks, createDocumentOutputBoundaryTasks } from "./cases/outputs.mjs";
import { createMachineProtocolTasks } from "./cases/machine-commands.mjs";
import { createProcessBoundaryCorpusTasks } from "./cases/corpus.mjs";
import { createCliArgumentCompatibilityWarningTasks, createCliArgumentFailureTasks } from "./cases/cli-args.mjs";
import { createOperationErrorTasks } from "./cases/operation-errors.mjs";
import { createInvokeFailureTasks } from "./cases/invoke-errors.mjs";

let suiteFailure = null;

try {
  smokeState.validators = compileSchemas();

  assertSetup(smokeState.binaryPath, "docnav-markdown binary path is required; pass --bin <path>");
  assertSetup(fs.existsSync(smokeState.binaryPath), `docnav-markdown binary not found: ${smokeState.binaryPath}`);
  assertSetup(fs.existsSync(fixturesDir), `fixture directory not found: ${fixturesDir}`);

  const results = await runSmokeTasks([
    { id: "document-link-chain", label: "Markdown document operation link chain", tasks: createDocumentLinkTasks() },
    { id: "document-output-boundary", label: "Markdown document output boundary", tasks: createDocumentOutputBoundaryTasks() },
    { id: "machine-protocol", label: "manifest probe and invoke protocol", tasks: createMachineProtocolTasks() },
    { id: "process-boundary-corpus", label: "Markdown process boundary corpus representative", tasks: createProcessBoundaryCorpusTasks() },
    { id: "cli-argument-failure", label: "CLI argument validation representative", tasks: createCliArgumentFailureTasks() },
    {
      id: "cli-argument-compatibility",
      label: "CLI argument compatibility warning representative",
      tasks: createCliArgumentCompatibilityWarningTasks()
    },
    { id: "operation-errors", label: "operation error mapping representative", tasks: createOperationErrorTasks() },
    { id: "invoke-failure", label: "invoke invalid request representative", tasks: createInvokeFailureTasks() }
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
