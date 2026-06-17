import { createProject } from "../fixtures.mjs";
import { runCli, validateSchema } from "../harness.mjs";
import {
  expectExit,
  expectNoJsonPayloadInStderr,
  expectProtocolFailure,
  parseJson
} from "../assertions.mjs";
import { exitCodes } from "../config.mjs";

export function createCliArgumentFailureTasks() {
  return [
    {
      id: "CORE-ARGS-001",
      label: "CORE-ARGS-001 strict CLI argument failure",
      run: testMissingRefProtocolFailure
    }
  ];
}

async function testMissingRefProtocolFailure() {
  const project = createProject("cli-argument-missing-ref");
  const record = await runCli("CORE-ARGS-001 read missing ref protocol-json parse failure", [
    "read",
    project.normalRelPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.input);
  expectNoJsonPayloadInStderr(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolFailure(record, json, "read", "INVALID_REQUEST");
}
