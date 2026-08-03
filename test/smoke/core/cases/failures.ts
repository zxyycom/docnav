import path from "node:path";

import {
  createProject,
  writeDamagedRegistry
} from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectProtocolFailure,
  parseJson
} from "../assertions.ts";
import { exitCodes } from "../config.ts";

export function createRegistryAndContractFailureTasks() {
  return [
    {
      id: "CORE-FAIL-001",
      label: "CORE-FAIL-001 lexical pathname routing miss",
      run: testLexicalPathnameRoutingMiss
    },
    {
      id: "CORE-SOURCE-001",
      label: "CORE-SOURCE-001 historical registry ignored",
      run: testHistoricalRegistryIgnored
    }
  ];
}

async function testLexicalPathnameRoutingMiss() {
  const project = createProject("failure-pathname-no-match");
  const documentPath = "docs/noextension";
  const routingPathname = path.join(project.root, documentPath).replaceAll(path.sep, "/");

  const record = await runCli("CORE-FAIL-001 missing extensionless path stops at lexical routing", [
    "outline",
    documentPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.documentRefFormat);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  const error = expectProtocolFailure(record, json, "outline", "FORMAT_UNKNOWN");
  const details = expectJsonObject(record, error.details, "FORMAT_UNKNOWN details are an object");
  expect(
    record,
    Object.keys(details).sort().join(",") === "candidates,path,reason",
    "FORMAT_UNKNOWN details contain only path, reason, and candidates"
  );
  expect(record, details.path === routingPathname, "FORMAT_UNKNOWN preserves the lexical routing pathname");
  expect(record, details.reason === "FORMAT_NOT_RECOGNIZED", "FORMAT_UNKNOWN identifies a pathname hint miss");
  expect(
    record,
    Array.isArray(details.candidates) && details.candidates.length === 0,
    "FORMAT_UNKNOWN carries an empty candidates array"
  );
}

async function testHistoricalRegistryIgnored() {
  const project = createProject("historical-registry-ignored");
  writeDamagedRegistry(project);

  const record = await runCli("CORE-SOURCE-001 damaged historical registry is ignored", [
    "outline",
    project.normalRelPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, 0);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expect(record, json.ok === true, "historical registry file does not affect built-in adapter dispatch");
}
