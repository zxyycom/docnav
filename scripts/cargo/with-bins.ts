import path from "node:path";
import { fileURLToPath } from "node:url";

import { runProcessSync } from "../tools/process.ts";
import { parseWithBinsArgs } from "./with-bins/args.ts";
import { buildCargoBins, environmentWithCargoBins } from "./with-bins/binaries.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const options = parseWithBinsArgs(process.argv.slice(2));
const executables = buildCargoBins(root, options.binaries, options.quiet);
const result = runProcessSync(options.command[0], options.command.slice(1), {
  cwd: root,
  env: environmentWithCargoBins(options.binaries, executables),
  stdio: "inherit"
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);
