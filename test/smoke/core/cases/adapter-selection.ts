import {
  createFakeAdapter,
  createProject,
  readAdapterCalls,
  writeProjectConfig,
  writeRegistry
} from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import type { CommandRecord } from "../../../tools/smoke-harness.ts";
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
      label: "CORE-SELECT-001 declared adapter selection failure",
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
  expectSelectionFailureDetails(record, error.details, invalid.id, "explicit");

  const invalidCalls = readAdapterCalls(invalid);
  const selectedCalls = readAdapterCalls(selected);
  expect(record, invalidCalls.some((call) => call.command === "manifest"), "invalid preselected adapter manifest was called");
  expect(record, selectedCalls.every((call) => call.command !== "probe"), "fallback adapter probe was not called");
  expect(record, selectedCalls.every((call) => call.command !== "invoke"), "fallback adapter invoke was not called");

  const configProject = createProject("selection-config-failure");
  const configInvalid = createFakeAdapter(configProject, { id: "fake-config-invalid-manifest", mode: "manifest-invalid" });
  const configSelected = createFakeAdapter(configProject, { id: "fake-config-after-invalid" });
  writeProjectConfig(configProject, {
    defaults: {
      adapter: configInvalid.id
    }
  });
  writeRegistry(configProject, [configInvalid, configSelected]);

  const configRecord = await runCli("CORE-SELECT-001 invalid config adapter returns selection diagnostic", [
    "outline",
    configProject.normalRelPath,
    "--output",
    "protocol-json"
  ], { project: configProject });
  expectExit(configRecord, exitCodes.protocolOrAdapterProcess);
  expectNoJsonPayloadInStderr(configRecord);
  const configJson = parseJson(configRecord);
  validateSchema(configRecord, "protocolResponse", configJson);
  const configError = expectProtocolFailure(configRecord, configJson, "outline", "ADAPTER_UNAVAILABLE");
  expectSelectionFailureDetails(configRecord, configError.details, configInvalid.id, "project");

  const configInvalidCalls = readAdapterCalls(configInvalid);
  const configSelectedCalls = readAdapterCalls(configSelected);
  expect(
    configRecord,
    configInvalidCalls.some((call) => call.command === "manifest"),
    "invalid config-selected adapter manifest was called"
  );
  expect(configRecord, configSelectedCalls.every((call) => call.command !== "probe"), "config fallback adapter probe was not called");
  expect(
    configRecord,
    configSelectedCalls.every((call) => call.command !== "invoke"),
    "config fallback adapter invoke was not called"
  );
}

function expectSelectionFailureDetails(
  record: CommandRecord,
  detailsValue: unknown,
  adapterId: string,
  source: string
) {
  const details = expectJsonObject(record, detailsValue, "selection failure details is object");
  expect(record, details.adapter_id === adapterId, "selection failure identifies declared adapter");
  expect(record, details.selection_source === source, `selection failure source is ${source}`);
  expect(record, details.stage === "resolve", "selection failure includes resolve stage");
  expect(
    record,
    typeof details.reason === "string" &&
      details.reason.includes("manifest") &&
      details.reason.includes("failed validation"),
    "selection failure reason describes manifest validation failure"
  );
}
