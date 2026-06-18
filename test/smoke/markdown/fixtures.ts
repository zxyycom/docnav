import fs from "node:fs";
import path from "node:path";

import { fixturesDir } from "./config.ts";
import { runCli, smokeState, validateSchema } from "./harness.ts";
import {
  assertSetup,
  expectExit,
  expectObjectArray,
  expectStderrEmpty,
  expectString,
  parseJson
} from "./assertions.ts";

export function fixture(name: string) {
  const filePath = path.join(fixturesDir, name);
  assertSetup(fs.existsSync(filePath), `missing fixture: ${filePath}`);
  return filePath;
}

export function setNormalRef(ref: string) {
  smokeState.normalRef = ref;
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
  const entries = expectObjectArray(record, json.entries, "normal outline entries are objects");
  smokeState.normalRef = expectString(record, entries[0]?.ref, "normal outline first ref is a string");
  return smokeState.normalRef;
}
