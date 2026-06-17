import { fixture } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectNoJsonPayloadInStderr,
  expectNoProtocolEnvelope,
  expectProtocolFailure,
  expectStderrEmpty,
  parseJson
} from "../assertions.ts";
import { exitCodes } from "../config.ts";

export function createOperationErrorTasks() {
  return [
    {
      id: "MD-ERROR-001",
      label: "MD-ERROR-001 markdown ref error output mapping",
      run: testRefErrorOutputMapping
    }
  ];
}

async function testRefErrorOutputMapping() {
  const normal = fixture("normal.md");
  const ref = "bad:ref";

  const readable = await runCli("MD-ERROR-001 invalid ref readable-json", [
    "read",
    normal,
    "--ref",
    ref,
    "--output",
    "readable-json"
  ]);
  expectExit(readable, exitCodes.documentRefFormat);
  expectStderrEmpty(readable);
  const readableJson = parseJson(readable);
  validateSchema(readable, "readableError", readableJson);
  expectNoProtocolEnvelope(readable, readableJson);
  expect(readable, readableJson.code === "REF_INVALID", "readable-json returns REF_INVALID");
  expect(readable, readableJson.details.ref === ref, "readable-json preserves details.ref");
  expect(readable, Array.isArray(readableJson.guidance), "readable-json includes guidance array");

  const protocol = await runCli("MD-ERROR-001 invalid ref protocol-json", [
    "read",
    normal,
    "--ref",
    ref,
    "--output",
    "protocol-json"
  ]);
  expectExit(protocol, exitCodes.documentRefFormat);
  expectNoJsonPayloadInStderr(protocol);
  const protocolJson = parseJson(protocol);
  validateSchema(protocol, "protocolResponse", protocolJson);
  expectProtocolFailure(protocol, protocolJson, "read", "REF_INVALID");
  expect(protocol, protocolJson.error.details.ref === ref, "protocol-json preserves error.details.ref");
}
