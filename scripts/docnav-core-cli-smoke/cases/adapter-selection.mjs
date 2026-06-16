import {
  createFakeAdapter,
  createProject,
  readAdapterCalls,
  copyNormalDocument,
  writeProjectConfig,
  writeRegistry
} from "../fixtures.mjs";
import { runCli, validateSchema } from "../harness.mjs";
import {
  expect,
  expectCandidateEvidence,
  expectCandidateWarning,
  expectExit,
  expectNoJsonPayloadInStderr,
  expectNoProtocolEnvelope,
  expectProtocolFailure,
  expectProtocolSuccess,
  expectStderrIncludes,
  expectStderrEmpty,
  parseJson
} from "../assertions.mjs";
import { exitCodes } from "../config.mjs";

export function createAdapterSelectionTasks() {
  return [
    { id: "core-adapter-selection-explicit", run: testExplicitAdapterPreselection },
    { id: "core-adapter-selection-config", run: testConfigAdapterPreselection },
    { id: "core-adapter-selection-extension", run: testExtensionInferencePreselection },
    { id: "core-adapter-selection-extension-invalid-continues", run: testExtensionInferenceContractFailureContinues },
    { id: "core-adapter-selection-supported-false-continues", run: testSupportedFalseContinues },
    { id: "core-adapter-selection-preselected-contract-mismatch", run: testPreselectedContractMismatchContinues },
    { id: "core-adapter-selection-candidate-evidence-all-failure", run: testCandidateEvidenceOnAllFailure },
    { id: "core-adapter-selection-registry-contract-failure", run: testRegistryTraversalContractFailureContinues },
    { id: "core-adapter-selection-protocol-warning-stderr", run: testProtocolJsonCandidateWarningUsesStderr }
  ];
}

async function testExplicitAdapterPreselection() {
  const project = createProject("selection-explicit");
  const fake = createFakeAdapter(project, { id: "fake-explicit" });
  writeRegistry(project, [fake]);

  const record = await runCli("explicit --adapter preselects fake adapter", [
    "outline",
    project.normalRelPath,
    "--adapter",
    fake.id,
    "--output",
    "readable-json"
  ], { project });
  const json = expectReadableOutline(record);
  expect(record, json.entries[0].display.includes(fake.id), "explicit adapter output is selected");
  const calls = readAdapterCalls(fake);
  expect(record, calls.some((call) => call.command === "manifest"), "explicit adapter manifest was called");
  expect(record, calls.some((call) => call.command === "probe"), "explicit adapter probe was called");
  expect(record, calls.some((call) => call.command === "invoke"), "explicit adapter invoke was called");
}

async function testConfigAdapterPreselection() {
  const project = createProject("selection-config");
  const fake = createFakeAdapter(project, { id: "fake-config" });
  writeProjectConfig(project, {
    defaults: {
      adapter: fake.id,
      limit_chars: 222,
      output: "readable-json"
    }
  });
  writeRegistry(project, [fake]);

  const record = await runCli("defaults.adapter preselects fake adapter", [
    "outline",
    project.normalRelPath
  ], { project });
  const json = expectReadableOutline(record);
  expect(record, json.entries[0].display.includes(fake.id), "config adapter output is selected");
  const invoke = readAdapterCalls(fake).find((call) => call.command === "invoke");
  expect(record, invoke?.stdin?.arguments?.limit_chars === 222, "project defaults.limit_chars reaches invoke request");
  expect(record, invoke?.stdin?.arguments?.page === 1, "omitted page reaches invoke request as 1");
}

async function testExtensionInferencePreselection() {
  const project = createProject("selection-extension");
  const docPath = copyNormalDocument(project, "docs/inferred.core");
  const fake = createFakeAdapter(project, { id: "fake-inferred", extensions: [".core"] });
  writeRegistry(project, [fake]);

  const record = await runCli("extension inference preselects fake adapter", [
    "outline",
    docPath,
    "--output",
    "readable-json"
  ], { project });
  const json = expectReadableOutline(record);
  expect(record, json.entries[0].display.includes(fake.id), "extension-inferred adapter output is selected");
}

async function testExtensionInferenceContractFailureContinues() {
  const project = createProject("selection-extension-invalid-continues");
  const docPath = copyNormalDocument(project, "docs/inferred.core");
  const invalid = createFakeAdapter(project, {
    id: "fake-invalid-extension",
    mode: "manifest-invalid",
    extensions: [".core"]
  });
  const selected = createFakeAdapter(project, { id: "fake-inferred-after-invalid", extensions: [".core"] });
  writeRegistry(project, [invalid, selected]);

  const record = await runCli("extension inference skips invalid manifest and continues", [
    "outline",
    docPath,
    "--output",
    "readable-json"
  ], { project });
  const json = expectReadableOutline(record);
  expect(record, json.entries[0].display.includes(selected.id), "adapter after invalid inferred candidate is selected");
  expect(record, readAdapterCalls(invalid).some((call) => call.command === "manifest"), "invalid inferred adapter manifest was called");
  expect(record, readAdapterCalls(selected).some((call) => call.command === "invoke"), "fallback inferred adapter invoke was called");
  expectCandidateWarning(record, json.warnings?.[0], {
    adapter_id: invalid.id,
    stage: "resolve",
    code: "MANIFEST_INVALID"
  });
}

async function testSupportedFalseContinues() {
  const project = createProject("selection-unsupported-continues");
  const unsupported = createFakeAdapter(project, { id: "fake-unsupported", mode: "probe-unsupported" });
  const selected = createFakeAdapter(project, { id: "fake-selected" });
  writeRegistry(project, [unsupported, selected]);

  const record = await runCli("supported false candidate continues to next adapter", [
    "outline",
    project.normalRelPath,
    "--output",
    "readable-json"
  ], { project });
  const json = expectReadableOutline(record);
  expect(record, json.entries[0].display.includes(selected.id), "adapter after unsupported probe is selected");
  expect(
    record,
    readAdapterCalls(unsupported).some((call) => call.command === "probe"),
    "unsupported adapter probe was called"
  );
  expect(
    record,
    !readAdapterCalls(unsupported).some((call) => call.command === "invoke"),
    "unsupported adapter invoke was not called"
  );
  expect(record, readAdapterCalls(selected).some((call) => call.command === "invoke"), "fallback adapter invoke was called");
  expectCandidateWarning(record, json.warnings?.[0], {
    adapter_id: unsupported.id,
    stage: "probe",
    code: "PROBE_UNSUPPORTED"
  });
}

async function testPreselectedContractMismatchContinues() {
  const project = createProject("selection-preselected-invalid-continues");
  const invalid = createFakeAdapter(project, { id: "fake-invalid-manifest", mode: "manifest-invalid" });
  const selected = createFakeAdapter(project, { id: "fake-after-invalid" });
  writeRegistry(project, [invalid, selected]);

  const record = await runCli("invalid explicit adapter continues with warning", [
    "outline",
    project.normalRelPath,
    "--adapter",
    invalid.id,
    "--output",
    "readable-json"
  ], { project });
  const json = expectReadableOutline(record);
  expect(record, json.entries[0].display.includes(selected.id), "adapter after invalid preselection is selected");
  expect(record, json.warnings[0].reason.includes("preselected adapter was not used"), "warning explains preselected failure");
  expectCandidateWarning(record, json.warnings?.[0], {
    adapter_id: invalid.id,
    stage: "resolve",
    code: "MANIFEST_INVALID",
    preselected: true
  });
}

async function testCandidateEvidenceOnAllFailure() {
  const project = createProject("selection-all-failed");
  const unsupported = createFakeAdapter(project, { id: "fake-unsupported-only", mode: "probe-unsupported" });
  writeRegistry(project, [unsupported]);

  const record = await runCli("format unknown includes candidate evidence", [
    "outline",
    project.normalRelPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.documentRefFormat);
  expectNoJsonPayloadInStderr(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolFailure(record, json, "outline", "FORMAT_UNKNOWN");
  const candidates = json.error.details.candidates;
  expect(record, Array.isArray(candidates), "FORMAT_UNKNOWN candidates is an array");
  expectCandidateEvidence(record, candidates[0], {
    adapter_id: unsupported.id,
    stage: "probe",
    code: "PROBE_UNSUPPORTED"
  });
}

async function testRegistryTraversalContractFailureContinues() {
  const project = createProject("selection-registry-contract-continues");
  const docPath = copyNormalDocument(project, "docs/noextension");
  const invalid = createFakeAdapter(project, { id: "fake-invalid-probe", mode: "probe-invalid" });
  const selected = createFakeAdapter(project, { id: "fake-after-invalid-probe" });
  writeRegistry(project, [invalid, selected]);

  const record = await runCli("registry traversal probe contract failure continues", [
    "outline",
    docPath,
    "--output",
    "readable-json"
  ], { project });
  const json = expectReadableOutline(record);
  expect(record, json.entries[0].display.includes(selected.id), "adapter after probe contract failure is selected");
  expect(record, readAdapterCalls(invalid).some((call) => call.command === "probe"), "invalid adapter probe was called");
  expect(record, readAdapterCalls(selected).some((call) => call.command === "invoke"), "registry traversal continued after contract failure");
  expectCandidateWarning(record, json.warnings?.[0], {
    adapter_id: invalid.id,
    stage: "probe",
    code: "PROBE_INVALID"
  });
}

async function testProtocolJsonCandidateWarningUsesStderr() {
  const project = createProject("selection-protocol-warning-stderr");
  const invalid = createFakeAdapter(project, { id: "fake-invalid-protocol-warning", mode: "manifest-invalid" });
  const selected = createFakeAdapter(project, { id: "fake-after-protocol-warning" });
  writeRegistry(project, [invalid, selected]);

  const record = await runCli("protocol-json candidate warning stays on stderr", [
    "outline",
    project.normalRelPath,
    "--adapter",
    invalid.id,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, 0);
  expectStderrIncludes(record, "id=adapter_candidate_failure");
  expectStderrIncludes(record, "effect=candidate_skipped");
  expectStderrIncludes(record, `"adapter_id":"${invalid.id}"`);
  expectStderrIncludes(record, "\"stage\":\"resolve\"");
  expectStderrIncludes(record, "\"code\":\"MANIFEST_INVALID\"");
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolSuccess(record, json, "outline");
}

function expectReadableOutline(record) {
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "readableOutline", json);
  expectNoProtocolEnvelope(record, json);
  expect(record, Array.isArray(json.entries) && json.entries.length > 0, "outline returns entries");
  return json;
}
