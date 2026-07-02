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

import {
  createRealMarkdownLinkTasks,
  createRealMarkdownRefErrorTasks
} from "./smoke/core/cases/real-markdown.ts";
import { createDocumentOutputBoundaryTasks } from "./smoke/core/cases/outputs.ts";
import { createAdapterSelectionTasks } from "./smoke/core/cases/adapter-selection.ts";
import { createCliArgumentFailureTasks } from "./smoke/core/cases/cli-args.ts";
import { createConfigContextTasks, createToolCommandTasks } from "./smoke/core/cases/config-management.ts";
import { createRegistryAndContractFailureTasks } from "./smoke/core/cases/failures.ts";

let suiteFailure;

try {
  smokeState.validators = compileSchemas();

  assertSetup(smokeState.docnavBinaryPath, "docnav binary path is required; pass --bin <path> or DOCNAV_BIN");
  const docnavBinaryPath = String(smokeState.docnavBinaryPath);
  assertSetup(fs.existsSync(docnavBinaryPath), `docnav binary not found: ${docnavBinaryPath}`);
  fs.mkdirSync(tempRoot, { recursive: true });

  const results = await runSmokeTasks([
    {
      id: "real-markdown-link-chain",
      label: "built-in markdown ref handoff chain",
      tasks: createRealMarkdownLinkTasks()
    },
    {
      id: "real-markdown-ref-error",
      label: "built-in markdown ref error mapping",
      tasks: createRealMarkdownRefErrorTasks()
    },
    { id: "document-output-boundary", label: "document output boundary", tasks: createDocumentOutputBoundaryTasks() },
    { id: "adapter-selection", label: "adapter selection representative", tasks: createAdapterSelectionTasks() },
    { id: "cli-argument-failure", label: "CLI argument failure representative", tasks: createCliArgumentFailureTasks() },
    {
      id: "config-context",
      label: "config precedence and path context",
      tasks: createConfigContextTasks()
    },
    {
      id: "registry-contract-failures",
      label: "registry and adapter contract failure representatives",
      tasks: createRegistryAndContractFailureTasks()
    },
    {
      id: "tool-commands",
      label: "init version doctor and help commands",
      tasks: createToolCommandTasks()
    }
  ]);
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
