import {
  createFakeAdapter,
  createProject,
  readAdapterCalls,
  writeRegistry
} from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectCandidateWarning,
  expectExit,
  expectObjectArray,
  expectNoProtocolEnvelope,
  expectStderrEmpty,
  expectString,
  parseJson
} from "../assertions.ts";

export function createAdapterSelectionTasks() {
  return [
    // @case BB-CORE-SELECT-001
    {
      id: "CORE-SELECT-001",
      label: "CORE-SELECT-001 adapter preselection fallback",
      run: testPreselectedAdapterFailureFallsBack
    }
  ];
}

async function testPreselectedAdapterFailureFallsBack() {
  const project = createProject("selection-preselected-fallback");
  const invalid = createFakeAdapter(project, { id: "fake-invalid-manifest", mode: "manifest-invalid" });
  const selected = createFakeAdapter(project, { id: "fake-after-invalid" });
  writeRegistry(project, [invalid, selected]);

  const record = await runCli("CORE-SELECT-001 invalid explicit adapter falls back with warning", [
    "outline",
    project.normalRelPath,
    "--adapter",
    invalid.id,
    "--output",
    "readable-json"
  ], { project });
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "readableOutline", json);
  expectNoProtocolEnvelope(record, json);
  const entries = expectObjectArray(record, json.entries, "outline entries are objects");
  const firstDisplay = expectString(record, entries[0]?.display, "first outline entry has display");
  expect(record, firstDisplay.includes(selected.id), "adapter after invalid preselection is selected");
  const warnings = expectObjectArray(record, json.warnings, "outline warnings are objects");
  expectCandidateWarning(record, warnings[0], {
    adapter_id: invalid.id,
    stage: "resolve",
    code: "MANIFEST_INVALID",
    preselected: true
  });

  const invalidCalls = readAdapterCalls(invalid);
  const selectedCalls = readAdapterCalls(selected);
  expect(record, invalidCalls.some((call) => call.command === "manifest"), "invalid preselected adapter manifest was called");
  expect(record, selectedCalls.some((call) => call.command === "probe"), "fallback adapter probe was called");
  expect(record, selectedCalls.some((call) => call.command === "invoke"), "fallback adapter invoke was called");
}
