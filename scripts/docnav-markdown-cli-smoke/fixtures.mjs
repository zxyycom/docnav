import fs from "node:fs";
import path from "node:path";

import { fixturesDir } from "./config.mjs";
import { runCli, smokeState, validateSchema } from "./harness.mjs";
import {
  assertSetup,
  expectExit,
  expectStderrEmpty,
  parseJson
} from "./assertions.mjs";

export function fixture(name) {
  const filePath = path.join(fixturesDir, name);
  assertSetup(fs.existsSync(filePath), `missing fixture: ${filePath}`);
  return filePath;
}

export function setNormalRef(ref) {
  smokeState.normalRef = ref;
}

export function getNormalRef() {
  if (smokeState.normalRef) {
    return smokeState.normalRef;
  }
  const normal = fixture("normal.md");
  const record = runCli("outline normal readable-json for ref", [
    "outline",
    normal,
    "--output",
    "readable-json"
  ]);
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "readableOutline", json);
  smokeState.normalRef = json.entries[0].ref;
  return smokeState.normalRef;
}
