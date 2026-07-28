import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const entrypoint = fileURLToPath(new URL("./index.ts", import.meta.url));
const workspaceRoot = path.resolve(path.dirname(entrypoint), "..", "..");

test("returns a query failure status at the CLI boundary", () => {
  const result = spawnSync(process.execPath, [
    entrypoint,
    "show",
    "CASE-THAT-DOES-NOT-EXIST",
    "--root",
    workspaceRoot
  ], {
    cwd: workspaceRoot,
    encoding: "utf8"
  });

  assert.equal(result.status, 6);
  assert.equal(result.stderr, "");
  assert.match(result.stdout, /"code": "query\.case-not-found"/);
});
