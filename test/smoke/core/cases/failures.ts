import {
  createProject,
  copyNormalDocument,
  writeDamagedRegistry
} from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectFormatCandidate,
  expectExit,
  expectJsonObject,
  expectObjectArray,
  expectProtocolFailure,
  parseJson
} from "../assertions.ts";
import { exitCodes } from "../config.ts";

export function createRegistryAndContractFailureTasks() {
  return [
    // @case BB-CORE-FAIL-001
    {
      id: "CORE-FAIL-001",
      label: "CORE-FAIL-001 adapter candidate failure summary",
      run: testCandidateFailureSummary
    },
    // @case BB-CORE-SOURCE-001
    {
      id: "CORE-SOURCE-001",
      label: "CORE-SOURCE-001 historical registry ignored",
      run: testHistoricalRegistryIgnored
    }
  ];
}

async function testCandidateFailureSummary() {
  const project = createProject("failure-candidate-evidence");
  const docPath = copyNormalDocument(project, "docs/noextension");

  const record = await runCli("CORE-FAIL-001 unsupported built-in candidate records summary", [
    "outline",
    docPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.documentRefFormat);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolFailure(record, json, "outline", "FORMAT_UNKNOWN");
  const error = expectJsonObject(record, json.error, "protocol error is an object");
  const details = expectJsonObject(record, error.details, "protocol error details is an object");
  expect(record, details.reason === "NO_SUPPORTED_ADAPTER", "FORMAT_UNKNOWN reason identifies unsupported adapter set");
  const candidates = expectObjectArray(record, details.candidates, "FORMAT_UNKNOWN candidates are objects");
  const candidate = candidates[0];
  expectFormatCandidate(record, candidate, {
    adapter_id: "docnav-markdown",
    stage: "probe",
    reason: "PROBE_UNSUPPORTED"
  });
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
