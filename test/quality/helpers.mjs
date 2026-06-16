import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";

export function runGit(cwd, args) {
  const result = spawnSync("git", args, {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.status, 0, `${args.join(" ")}\n${result.stdout}\n${result.stderr}`);
  return result;
}
