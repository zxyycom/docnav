import fs from "node:fs";

import { fixturesDir } from "./smoke/markdown/config.ts";
import { assertSetup } from "./smoke/markdown/assertions.ts";
import {
  compileSchemas,
  printFailureSummary,
  printSuccessSummary,
  runSmokeTasks,
  smokeState,
  writeAuditLogs
} from "./smoke/markdown/harness.ts";

import { createDocumentLinkTasks, createDocumentOutputBoundaryTasks } from "./smoke/markdown/cases/outputs.ts";
import { createMachineProtocolTasks } from "./smoke/markdown/cases/machine-commands.ts";
import { createProcessBoundaryCorpusTasks } from "./smoke/markdown/cases/corpus.ts";
import { createMarkdownConfigTasks } from "./smoke/markdown/cases/config.ts";
import {
  createCliArgumentCompatibilityWarningTasks,
  createCliArgumentFailureTasks
} from "./smoke/markdown/cases/cli-args.ts";
import { createOperationErrorTasks } from "./smoke/markdown/cases/operation-errors.ts";
import { createInvokeFailureTasks } from "./smoke/markdown/cases/invoke-errors.ts";

let suiteFailure;

try {
  smokeState.validators = compileSchemas();

  assertSetup(smokeState.binaryPath, "docnav-markdown binary path is required; pass --bin <path>");
  const binaryPath = String(smokeState.binaryPath);
  assertSetup(fs.existsSync(binaryPath), `docnav-markdown binary not found: ${binaryPath}`);
  assertSetup(fs.existsSync(fixturesDir), `fixture directory not found: ${fixturesDir}`);

  const results = await runSmokeTasks([
    { id: "document-link-chain", label: "Markdown document operation link chain", tasks: createDocumentLinkTasks() },
    { id: "document-output-boundary", label: "Markdown document output boundary", tasks: createDocumentOutputBoundaryTasks() },
    { id: "machine-protocol", label: "manifest probe and invoke protocol", tasks: createMachineProtocolTasks() },
    { id: "process-boundary-corpus", label: "Markdown process boundary corpus representative", tasks: createProcessBoundaryCorpusTasks() },
    { id: "direct-cli-config", label: "Markdown direct CLI config precedence and boundaries", tasks: createMarkdownConfigTasks() },
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
