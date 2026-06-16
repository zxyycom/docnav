import fs from "node:fs";

import { tempRoot } from "./config.mjs";
import { assertSetup } from "./assertions.mjs";
import {
  compileSchemas,
  printFailureSummary,
  printSuccessSummary,
  runSmokeTasks,
  smokeState,
  writeAuditLogs
} from "./harness.mjs";

import {
  createRealMarkdownFindRefReadTasks,
  createRealMarkdownOutlineRefReadTasks,
  createRealMarkdownRefInvalidTasks,
  createRealMarkdownRefNotFoundTasks
} from "./cases/real-markdown.mjs";
import { createDocumentOutputMatrixTasks } from "./cases/outputs.mjs";
import { createAdapterSelectionTasks } from "./cases/adapter-selection.mjs";
import { createCliArgumentFailureTasks } from "./cases/cli-args.mjs";
import { createConfigContextAndCompatibilityTasks } from "./cases/config-management.mjs";
import { createRegistryAndContractFailureTasks } from "./cases/failures.mjs";

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

  const results = await runSmokeTasks([
    {
      id: "real-markdown-outline-read",
      label: "real docnav + real docnav-markdown outline -> ref -> read",
      tasks: createRealMarkdownOutlineRefReadTasks()
    },
    {
      id: "real-markdown-find-read",
      label: "real docnav + real docnav-markdown find -> ref -> read",
      tasks: createRealMarkdownFindRefReadTasks()
    },
    {
      id: "real-markdown-ref-invalid",
      label: "real docnav + real docnav-markdown REF_INVALID mapping",
      tasks: createRealMarkdownRefInvalidTasks()
    },
    {
      id: "real-markdown-ref-not-found",
      label: "real docnav + real docnav-markdown REF_NOT_FOUND mapping",
      tasks: createRealMarkdownRefNotFoundTasks()
    },
    { id: "document-output-matrix", label: "document operation output matrix", tasks: createDocumentOutputMatrixTasks() },
    { id: "adapter-selection", label: "adapter selection matrix", tasks: createAdapterSelectionTasks() },
    { id: "cli-argument-failures", label: "CLI argument failure matrix", tasks: createCliArgumentFailureTasks() },
    {
      id: "config-context-compatibility",
      label: "config context and compatibility warnings",
      tasks: createConfigContextAndCompatibilityTasks()
    },
    {
      id: "registry-contract-failures",
      label: "registry and adapter contract failure matrix",
      tasks: createRegistryAndContractFailureTasks()
    }
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

fs.rmSync(tempRoot, { recursive: true, force: true });
printSuccessSummary();
