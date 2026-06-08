import {
  createFakeAdapter,
  createProject,
  writeDamagedRegistry,
  writeDocument,
  writeRegistry
} from "../fixtures.mjs";
import { runCli } from "../runner.mjs";
import {
  expect,
  expectCandidateEvidence,
  expectExit,
  expectProtocolFailure,
  parseJson
} from "../assertions.mjs";
import { validateSchema } from "../schemas.mjs";
import { exitCodes } from "../config.mjs";

export function testRegistryAndContractFailures() {
  testMissingRegistry();
  testDamagedRegistry();
  testInvalidRegistryCommandPath();
  testDuplicateAdapterId();
  testManifestContractFailure();
  testProbeContractFailure();
  testManifestProcessFailure();
  testProbeProcessFailure();
  testInvokeContractFailure();
  testInvokeProcessFailure();
}

function testMissingRegistry() {
  const project = createProject("failure-registry-missing");

  const record = runCli("missing registry returns FORMAT_UNKNOWN", [
    "outline",
    project.normalRelPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.documentRefFormat);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolFailure(record, json, "outline", "FORMAT_UNKNOWN");
  expect(record, Array.isArray(json.error.details.candidates), "missing registry candidates is an array");
  expect(record, json.error.details.candidates.length === 0, "missing registry has no candidates");
}

function testDamagedRegistry() {
  const project = createProject("failure-registry-damaged");
  writeDamagedRegistry(project);

  const record = runCli("damaged registry returns INVALID_REQUEST", [
    "outline",
    project.normalRelPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.input);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolFailure(record, json, "outline", "INVALID_REQUEST");
  expect(record, json.error.details.field === "adapter_registry", "damaged registry error identifies registry field");
}

function testInvalidRegistryCommandPath() {
  const project = createProject("failure-registry-command");
  writeRegistry(project, [{ id: "bad-command", command: "../adapter" }]);

  const record = runCli("invalid registry command path returns INVALID_REQUEST", [
    "outline",
    project.normalRelPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.input);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolFailure(record, json, "outline", "INVALID_REQUEST");
  expect(record, json.error.details.field === "adapter_registry.adapters[].command", "invalid command field is reported");
}

function testDuplicateAdapterId() {
  const project = createProject("failure-registry-duplicate");
  const first = createFakeAdapter(project, { id: "duplicate" });
  const second = createFakeAdapter(project, { id: "duplicate-second" });
  writeRegistry(project, [
    { id: "duplicate", command: first.command },
    { id: "duplicate", command: second.command }
  ]);

  const record = runCli("duplicate adapter id returns INVALID_REQUEST", [
    "outline",
    project.normalRelPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.input);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolFailure(record, json, "outline", "INVALID_REQUEST");
  expect(record, json.error.details.field === "adapter_registry.adapters[].id", "duplicate id field is reported");
}

function testManifestContractFailure() {
  const project = createProject("failure-manifest-contract");
  const docPath = writeDocument(project, "docs/noextension");
  const invalid = createFakeAdapter(project, { id: "fake-invalid-manifest", mode: "manifest-invalid" });
  writeRegistry(project, [invalid]);

  const record = runCli("manifest contract failure records candidate evidence", [
    "outline",
    docPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.documentRefFormat);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolFailure(record, json, "outline", "FORMAT_UNKNOWN");
  expectCandidateEvidence(record, json.error.details.candidates?.[0], {
    adapter_id: invalid.id,
    stage: "resolve",
    code: "MANIFEST_INVALID"
  });
}

function testProbeContractFailure() {
  const project = createProject("failure-probe-contract");
  const docPath = writeDocument(project, "docs/noextension");
  const invalid = createFakeAdapter(project, { id: "fake-invalid-probe", mode: "probe-invalid" });
  writeRegistry(project, [invalid]);

  const record = runCli("probe contract failure records candidate evidence", [
    "outline",
    docPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.documentRefFormat);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolFailure(record, json, "outline", "FORMAT_UNKNOWN");
  expectCandidateEvidence(record, json.error.details.candidates?.[0], {
    adapter_id: invalid.id,
    stage: "probe",
    code: "PROBE_INVALID"
  });
}

function testManifestProcessFailure() {
  const project = createProject("failure-manifest-process");
  const docPath = writeDocument(project, "docs/noextension");
  const failed = createFakeAdapter(project, { id: "fake-manifest-exit", mode: "manifest-exit" });
  writeRegistry(project, [failed]);

  const record = runCli("manifest process failure records candidate evidence", [
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
  expect(record, candidate.details.exit_code === 7, "manifest process evidence includes exit_code");
  expect(record, candidate.details.stderr.includes("manifest failed intentionally"), "manifest process evidence includes stderr");
}

function testProbeProcessFailure() {
  const project = createProject("failure-probe-process");
  const docPath = writeDocument(project, "docs/noextension");
  const failed = createFakeAdapter(project, { id: "fake-probe-exit", mode: "probe-exit" });
  writeRegistry(project, [failed]);

  const record = runCli("probe process failure records candidate evidence", [
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
    stage: "probe",
    code: "ADAPTER_UNAVAILABLE"
  });
  expect(record, candidate.details.exit_code === 8, "probe process evidence includes exit_code");
  expect(record, candidate.details.stderr.includes("probe failed intentionally"), "probe process evidence includes stderr");
}

function testInvokeContractFailure() {
  const project = createProject("failure-invoke-contract");
  const invalid = createFakeAdapter(project, { id: "fake-invalid-invoke", mode: "invoke-schema-invalid" });
  writeRegistry(project, [invalid]);

  const record = runCli("invoke contract failure returns ADAPTER_INVOKE_FAILED", [
    "outline",
    project.normalRelPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.protocolOrAdapterProcess);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolFailure(record, json, "outline", "ADAPTER_INVOKE_FAILED");
  expect(record, json.error.details.adapter_id === invalid.id, "invoke failure identifies adapter id");
}

function testInvokeProcessFailure() {
  const project = createProject("failure-invoke-process");
  const failed = createFakeAdapter(project, { id: "fake-invoke-exit", mode: "invoke-exit" });
  writeRegistry(project, [failed]);

  const record = runCli("invoke process failure returns ADAPTER_INVOKE_FAILED", [
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
