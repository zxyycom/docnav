import { assert } from "../tools/validators/assertions.ts";
import { TASK_NAMES } from "../tools/validators/config.ts";
import { validateMarkdownLinks } from "../tools/validators/links.ts";
import { validateMcpStructuredContent } from "../tools/validators/mcp.ts";
import { validateJsonSyntax, validateSchemas } from "../tools/validators/schema/index.ts";
import { validateOutputModeConsistency } from "../tools/validators/output/document-output-modes.ts";
import { validateProtocolExampleSemantics } from "../tools/validators/protocol/protocol-examples.ts";
import { validateTestCaseCatalog } from "../tools/validators/case-catalog/validator.ts";

const requested = new Set(process.argv.slice(2));
const runAll = requested.size === 0;

const tasks = {
  [TASK_NAMES.cases]: validateTestCaseCatalog,
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

function validateExampleConsistency() {
  validateProtocolExampleSemantics();
  validateOutputModeConsistency();
}
