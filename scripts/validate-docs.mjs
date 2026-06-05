import { assert } from "./validators/fs-utils.mjs";
import { TASK_NAMES } from "./validators/config.mjs";
import { validateMarkdownLinks } from "./validators/links.mjs";
import { validateMcpStructuredContent } from "./validators/mcp.mjs";
import { validateJsonSyntax, validateSchemas } from "./validators/schema.mjs";
import { validateExampleSemantics } from "./validators/semantics.mjs";

const requested = new Set(process.argv.slice(2));
const runAll = requested.size === 0;

const tasks = {
  [TASK_NAMES.json]: validateJsonSyntax,
  [TASK_NAMES.schema]: validateSchemas,
  [TASK_NAMES.mcp]: validateMcpStructuredContent,
  [TASK_NAMES.semantics]: validateExampleSemantics,
  [TASK_NAMES.links]: validateMarkdownLinks
};

const selectedTasks = runAll ? Object.keys(tasks) : [...requested];
for (const taskName of selectedTasks) {
  const task = tasks[taskName];
  assert(task, `unknown validation task: ${taskName}`);
  task();
}
