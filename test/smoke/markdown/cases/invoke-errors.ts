import { exitCodes } from "../config.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expectExit,
  expectNoJsonPayloadInStderr,
  expectProtocolFailure,
  expectStderrIncludes,
  parseJson
} from "../assertions.ts";

export function createInvokeFailureTasks() {
  return [
    {
      id: "MD-INVOKE-001",
      label: "MD-INVOKE-001 invoke invalid request failure",
      run: testInvokeMalformedJsonFailure
    }
  ];
}

async function testInvokeMalformedJsonFailure() {
  const record = await runCli("MD-INVOKE-001 invoke malformed JSON", ["invoke"], {
    stdin: "{ \"protocol_version\":",
    stdinSummary: "malformed JSON request"
  });
  expectExit(record, exitCodes.input);
  expectStderrIncludes(record, "invalid request JSON");
  expectNoJsonPayloadInStderr(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolFailure(record, json, null, "INVALID_REQUEST");
}
