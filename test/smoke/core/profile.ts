import type { SmokeTask } from "../../tools/smoke-harness.ts";

import {
  createRealMarkdownLinkTasks,
  createRealMarkdownRefErrorTasks
} from "./cases/real-markdown.ts";
import { createAutoReadTasks } from "./cases/auto-read.ts";
import { createDocumentOutputBoundaryTasks } from "./cases/outputs.ts";
import { createAdapterSelectionTasks } from "./cases/adapter-selection.ts";
import { createCliArgumentFailureTasks } from "./cases/cli-args.ts";
import {
  createConfigContextTasks,
  createToolCommandTasks
} from "./cases/config-management.ts";
import { createRegistryAndContractFailureTasks } from "./cases/failures.ts";

export function createCoreSmokeTasks(): SmokeTask[] {
  return [
    {
      id: "real-markdown-link-chain",
      label: "built-in Markdown navigation behavior",
      tasks: createRealMarkdownLinkTasks()
    },
    {
      id: "real-markdown-ref-error",
      label: "built-in markdown ref error mapping",
      tasks: createRealMarkdownRefErrorTasks()
    },
    {
      id: "auto-read",
      label: "unique-ref auto-read defaults and disable sources",
      tasks: createAutoReadTasks()
    },
    {
      id: "document-output-boundary",
      label: "document output boundary",
      tasks: createDocumentOutputBoundaryTasks()
    },
    {
      id: "adapter-selection",
      label: "adapter selection representative",
      tasks: createAdapterSelectionTasks()
    },
    {
      id: "cli-argument-failure",
      label: "CLI argument failure representative",
      tasks: createCliArgumentFailureTasks()
    },
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
  ];
}
