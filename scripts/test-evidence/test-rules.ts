import path from "node:path";
import { fileURLToPath } from "node:url";

import { writeProcessOutput } from "../tools/foundation/src/index.ts";
import {
  expectedAstGrepVersionLine,
  runAstGrep
} from "./ast-grep.ts";

const workspaceRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  ".."
);
const configPath = path.join(
  workspaceRoot,
  "scripts",
  "test-evidence",
  "sgconfig.yml"
);

const version = await runAstGrep(["--version"], { workspaceRoot });
writeProcessOutput(version);
if (version.status !== 0 || version.stdout.trim() !== expectedAstGrepVersionLine()) {
  throw new Error(
    `unexpected ast-grep version: status=${String(version.status)} stdout=${JSON.stringify(version.stdout)}`
  );
}

const result = await runAstGrep([
  "test",
  "--config",
  configPath,
  "--skip-snapshot-tests",
  "--color",
  "never"
], { workspaceRoot });
writeProcessOutput(result);
if (result.status !== 0) {
  throw new Error(`ast-grep rule tests failed with status ${String(result.status)}`);
}
