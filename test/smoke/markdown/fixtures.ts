import fs from "node:fs";
import path from "node:path";

import { fixturesDir } from "./config.ts";
import { runCli, smokeState, validateSchema } from "./harness.ts";
import {
  assertSetup,
  expectExit,
  expectStderrEmpty,
  parseJson
} from "./assertions.ts";

export function fixture(name: ExternalValue) {
  const filePath = path.join(fixturesDir, name);
  assertSetup(fs.existsSync(filePath), `missing fixture: ${filePath}`);
  return filePath;
}

export function setNormalRef(ref: ExternalValue) {
  smokeState.normalRef = String(ref);
}

export async function getNormalRef(): Promise<string> {
  if (smokeState.normalRef) {
    return smokeState.normalRef;
  }
  if (smokeState.normalRefPromise) {
    return smokeState.normalRefPromise;
  }
  smokeState.normalRefPromise = loadNormalRef();
  return smokeState.normalRefPromise;
}

async function loadNormalRef() {
  const normal = fixture("normal.md");
  const record = await runCli("outline normal readable-json for ref", [
    "outline",
    normal,
    "--output",
    "readable-json"
  ]);
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "readableOutline", json);
  smokeState.normalRef = String(json.entries[0].ref);
  return smokeState.normalRef;
}
