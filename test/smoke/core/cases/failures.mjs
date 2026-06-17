import {
  createFakeAdapter,
  createProject,
  copyNormalDocument,
  writeRegistry
} from "../fixtures.mjs";
import { runCli, validateSchema } from "../harness.mjs";
import {
  expect,
  expectCandidateEvidence,
  expectExit,
  expectProtocolFailure,
  parseJson
} from "../assertions.mjs";
import { exitCodes } from "../config.mjs";

export function createRegistryAndContractFailureTasks() {
  return [
    {
      id: "CORE-FAIL-001",
      label: "CORE-FAIL-001 adapter candidate failure evidence",
      run: testCandidateFailureEvidence
    },
    {
      id: "CORE-INVOKE-001",
      label: "CORE-INVOKE-001 adapter invoke process failure",
      run: testInvokeProcessFailure
    }
  ];
}

async function testCandidateFailureEvidence() {
  const project = createProject("failure-candidate-evidence");
  const docPath = copyNormalDocument(project, "docs/noextension");
  const failed = createFakeAdapter(project, { id: "fake-manifest-exit", mode: "manifest-exit" });
  writeRegistry(project, [failed]);

  const record = await runCli("CORE-FAIL-001 manifest process failure records candidate evidence", [
    "outline",
    docPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.documentRefFormat);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolFailure(record, json, "outline", "FORMAT_UNKNOWN");
  const candidate = json.error.details.candidates?.[0];
  expectCandidateEvidence(record, candidate, {
    adapter_id: failed.id,
    stage: "resolve",
    code: "ADAPTER_UNAVAILABLE"
  });
  expect(record, candidate.details.exit_code === 7, "candidate evidence includes exit_code");
  expect(record, candidate.details.stderr.includes("manifest failed intentionally"), "candidate evidence includes stderr");
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
  expect(record, json.error.details.adapter_id === failed.id, "invoke process failure identifies adapter id");
  expect(record, json.error.details.exit_code === 9, "invoke process failure includes exit_code");
  expect(record, json.error.details.stderr.includes("invoke failed intentionally"), "invoke process failure includes stderr");
}
