import path from "node:path";
import { fileURLToPath } from "node:url";

import { errorMessage } from "../tools/errors.ts";
import { parseArgs, resolveVerificationConcurrency } from "./verify/args.ts";
import { printUsage } from "./verify/output.ts";
import { runVerification } from "./verify/runner.ts";

export {
  PROFILE_FULL,
  PROFILE_REQUIRED,
  checks,
  checksForProfile,
  profiles,
  reportCountForChecks,
  visibleOutputLines,
  isIgnoredOutput
} from "./checks/index.ts";
export { formatCompletionLine, formatDurationMs } from "./results.ts";
export { parseArgs, resolveVerificationConcurrency };

if (isMainModule()) {
  void main();
}

async function main() {
  try {
    const options = parseArgs(process.argv.slice(2));
    if (options.help) {
      printUsage(console.log);
      process.exitCode = 0;
      return;
    }
    process.exitCode = await runVerification(options);
  } catch (error: unknown) {
    console.error(errorMessage(error));
    printUsage(console.error);
    process.exitCode = 2;
  }
}

function isMainModule() {
  return process.argv[1] ? path.resolve(process.argv[1]) === fileURLToPath(import.meta.url) : false;
}
