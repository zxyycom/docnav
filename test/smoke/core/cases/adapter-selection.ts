import { createProject } from "../fixtures.ts";
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
    details.reason === "ADAPTER_NOT_FOUND",
    "selection failure reason identifies the missing adapter"
  );
}
