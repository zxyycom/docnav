import {
  createProject,
  writeProjectConfig,
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
  const missingAdapter = "custom-local-adapter";

  const record = await runCli("CORE-SELECT-001 invalid explicit adapter returns selection diagnostic", [
    "outline",
    project.normalRelPath,
    "--adapter",
    missingAdapter,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.protocolOrAdapterProcess);
  expectNoJsonPayloadInStderr(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  const error = expectProtocolFailure(record, json, "outline", "ADAPTER_UNAVAILABLE");
  expectSelectionFailureDetails(record, error.details, missingAdapter, "explicit");

  const invalidNativeRecord = await runCli(
    "CORE-SELECT-001 invalid native option does not preempt missing adapter",
    [
      "outline",
      project.normalRelPath,
      "--adapter",
      missingAdapter,
      "--max-heading-level",
      "100",
      "--output",
      "protocol-json"
    ],
    { project }
  );
  expectExit(invalidNativeRecord, exitCodes.protocolOrAdapterProcess);
  expectNoJsonPayloadInStderr(invalidNativeRecord);
  const invalidNativeJson = parseJson(invalidNativeRecord);
  validateSchema(invalidNativeRecord, "protocolResponse", invalidNativeJson);
  const invalidNativeError = expectProtocolFailure(
    invalidNativeRecord,
    invalidNativeJson,
    "outline",
    "ADAPTER_UNAVAILABLE"
  );
  expectSelectionFailureDetails(
    invalidNativeRecord,
    invalidNativeError.details,
    missingAdapter,
    "explicit"
  );

  const configProject = createProject("selection-config-failure");
  const configMissingAdapter = "project-config-adapter";
  writeProjectConfig(configProject, {
    defaults: {
      adapter: configMissingAdapter
    }
  });

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
  expectSelectionFailureDetails(configRecord, configError.details, configMissingAdapter, "project");
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
      details.reason.includes("core release static registry"),
    "selection failure reason describes static registry miss"
  );
}
