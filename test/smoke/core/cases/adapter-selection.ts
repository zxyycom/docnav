import {
  createFakeAdapter,
  createProject,
  readAdapterCalls,
  writeRegistry
} from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectNoJsonPayloadInStderr,
  expectProtocolFailure,
  parseJson
} from "../assertions.ts";
import { exitCodes } from "../config.ts";

export function createAdapterSelectionTasks() {
  return [
    // @case BB-CORE-SELECT-001
    {
      id: "CORE-SELECT-001",
      label: "CORE-SELECT-001 explicit adapter selection failure",
      run: testExplicitAdapterFailureStopsSelection
    }
  ];
}

async function testExplicitAdapterFailureStopsSelection() {
  const project = createProject("selection-explicit-failure");
  const invalid = createFakeAdapter(project, { id: "fake-invalid-manifest", mode: "manifest-invalid" });
  const selected = createFakeAdapter(project, { id: "fake-after-invalid" });
  writeRegistry(project, [invalid, selected]);

  const record = await runCli("CORE-SELECT-001 invalid explicit adapter returns selection diagnostic", [
    "outline",
    project.normalRelPath,
    "--adapter",
    invalid.id,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.protocolOrAdapterProcess);
  expectNoJsonPayloadInStderr(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  const error = expectProtocolFailure(record, json, "outline", "ADAPTER_UNAVAILABLE");
  const details = expectJsonObject(record, error.details, "selection failure details is object");
  expect(record, details.adapter_id === invalid.id, "selection failure identifies explicit adapter");
  expect(
    record,
    typeof details.reason === "string" &&
      details.reason.includes("manifest") &&
      details.reason.includes("failed validation"),
    "selection failure reason describes manifest validation failure"
  );

  const invalidCalls = readAdapterCalls(invalid);
  const selectedCalls = readAdapterCalls(selected);
  expect(record, invalidCalls.some((call) => call.command === "manifest"), "invalid preselected adapter manifest was called");
  expect(record, selectedCalls.every((call) => call.command !== "probe"), "fallback adapter probe was not called");
  expect(record, selectedCalls.every((call) => call.command !== "invoke"), "fallback adapter invoke was not called");
}
