import fs from "node:fs";
import path from "node:path";

import { exitCodes, fixturesDir } from "./config.mjs";
import { smokeState } from "./state.mjs";
import { runCli } from "./runner.mjs";
import {
  assertSetup,
  expectExit,
  expectNormalFindResult,
  expectNoProtocolEnvelope,
  expectProtocolSuccess,
  expectReadResultsEquivalent,
  expectStderrEmpty,
  parseJson
} from "./assertions.mjs";
import { validateSchema } from "./schemas.mjs";

export function fixture(name) {
  const filePath = path.join(fixturesDir, name);
  assertSetup(fs.existsSync(filePath), `missing fixture: ${filePath}`);
  return filePath;
}

export function setNormalRef(ref) {
  smokeState.normalRef = ref;
}

export function setNormalReadableReadResult(result) {
  smokeState.normalReadableReadResult = result;
}

export function setNormalReadableFindResult(result) {
  smokeState.normalReadableFindResult = result;
}

export function setNormalProtocolReadResult(result) {
  smokeState.normalProtocolReadResult = result;
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

export function getNormalReadableReadResult() {
  if (smokeState.normalReadableReadResult) {
    return smokeState.normalReadableReadResult;
  }
  const normal = fixture("normal.md");
  const record = runCli("read normal readable-json for equivalence", [
    "read",
    normal,
    "--ref",
    getNormalRef(),
    "--output",
    "readable-json"
  ]);
  expectExit(record, exitCodes.success);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "readableRead", json);
  expectNoProtocolEnvelope(record, json);
  smokeState.normalReadableReadResult = json;
  return smokeState.normalReadableReadResult;
}

export function getNormalReadableFindResult() {
  if (smokeState.normalReadableFindResult) {
    return smokeState.normalReadableFindResult;
  }
  const normal = fixture("normal.md");
  const record = runCli("find normal readable-json for equivalence", [
    "find",
    normal,
    "--query",
    "target",
    "--output",
    "readable-json"
  ]);
  expectExit(record, exitCodes.success);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "readableFind", json);
  expectNoProtocolEnvelope(record, json);
  expectNormalFindResult(record, json, "readable find");
  smokeState.normalReadableFindResult = json;
  return smokeState.normalReadableFindResult;
}

export function getNormalProtocolReadResult() {
  if (smokeState.normalProtocolReadResult) {
    return smokeState.normalProtocolReadResult;
  }
  const normal = fixture("normal.md");
  const record = runCli("read normal protocol-json for equivalence", [
    "read",
    normal,
    "--ref",
    getNormalRef(),
    "--output",
    "protocol-json"
  ]);
  expectExit(record, exitCodes.success);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolSuccess(record, json, "read");
  expectReadResultsEquivalent(
    record,
    json.result,
    getNormalReadableReadResult(),
    "read protocol-json result matches readable-json"
  );
  smokeState.normalProtocolReadResult = json.result;
  return smokeState.normalProtocolReadResult;
}

