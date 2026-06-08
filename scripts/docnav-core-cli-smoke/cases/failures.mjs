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
  testInvokeContractFailure();
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

  const record = runCli("manifest contract failure returns ADAPTER_UNAVAILABLE", [
    "outline",
    docPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.protocolOrAdapterProcess);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolFailure(record, json, "outline", "ADAPTER_UNAVAILABLE");
  expect(record, json.error.details.adapter_id === invalid.id, "manifest failure identifies adapter id");
}

function testProbeContractFailure() {
  const project = createProject("failure-probe-contract");
  const docPath = writeDocument(project, "docs/noextension");
  const invalid = createFakeAdapter(project, { id: "fake-invalid-probe", mode: "probe-invalid" });
  writeRegistry(project, [invalid]);

  const record = runCli("probe contract failure returns ADAPTER_UNAVAILABLE", [
    "outline",
    docPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.protocolOrAdapterProcess);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolFailure(record, json, "outline", "ADAPTER_UNAVAILABLE");
  expect(record, json.error.details.adapter_id === invalid.id, "probe failure identifies adapter id");
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

