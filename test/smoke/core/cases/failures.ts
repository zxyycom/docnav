import {
  createFakeAdapter,
  createProject,
  copyNormalDocument,
  writeRegistry
} from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectFormatCandidate,
  expectExit,
  expectJsonObject,
  expectObjectArray,
  expectProtocolFailure,
  expectString,
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
    // @case BB-CORE-INVOKE-001
    {
      id: "CORE-INVOKE-001",
      label: "CORE-INVOKE-001 adapter invoke process failure",
      run: testInvokeProcessFailure
    }
  ];
}

async function testCandidateFailureSummary() {
  const project = createProject("failure-candidate-evidence");
  const docPath = copyNormalDocument(project, "docs/noextension");
  const failed = createFakeAdapter(project, { id: "fake-manifest-exit", mode: "manifest-exit" });
  writeRegistry(project, [failed]);

  const record = await runCli("CORE-FAIL-001 manifest process failure records candidate summary", [
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
    adapter_id: failed.id,
    stage: "resolve",
    reason: "ADAPTER_UNAVAILABLE"
  });
}

async function testInvokeProcessFailure() {
  const project = createProject("failure-invoke-process");
  const failed = createFakeAdapter(project, { id: "fake-invoke-exit", mode: "invoke-exit" });
  writeRegistry(project, [failed]);

  const record = await runCli("CORE-INVOKE-001 invoke process failure returns ADAPTER_INVOKE_FAILED", [
    "outline",
    project.normalRelPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.protocolOrAdapterProcess);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolFailure(record, json, "outline", "ADAPTER_INVOKE_FAILED");
  const error = expectJsonObject(record, json.error, "protocol error is an object");
  const details = expectJsonObject(record, error.details, "protocol error details is an object");
  const stderr = expectString(record, details.stderr, "invoke failure stderr is a string");
  expect(record, details.adapter_id === failed.id, "invoke process failure identifies adapter id");
  expect(record, details.exit_code === 9, "invoke process failure includes exit_code");
  expect(record, stderr.includes("invoke failed intentionally"), "invoke process failure includes stderr");
}
