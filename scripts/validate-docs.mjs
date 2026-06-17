import { assert } from "./tools/validators/fs-utils.mjs";
import { TASK_NAMES } from "./tools/validators/config.mjs";
import { validateMarkdownLinks } from "./tools/validators/links.mjs";
import { validateMcpStructuredContent } from "./tools/validators/mcp.mjs";
import { validateJsonSyntax, validateSchemas } from "./tools/validators/schema.mjs";
import { validateExampleConsistency } from "./tools/validators/example-consistency.mjs";

const requested = new Set(process.argv.slice(2));
const runAll = requested.size === 0;

const tasks = {
  [TASK_NAMES.json]: validateJsonSyntax,
  [TASK_NAMES.schema]: validateSchemas,
  [TASK_NAMES.mcp]: validateMcpStructuredContent,
  [TASK_NAMES.examples]: validateExampleConsistency,
  [TASK_NAMES.links]: validateMarkdownLinks
};

const selectedTasks = runAll ? Object.keys(tasks) : [...requested];
for (const taskName of selectedTasks) {
  const task = tasks[taskName];
  assert(task, `unknown validation task: ${taskName}`);
  task();
}
