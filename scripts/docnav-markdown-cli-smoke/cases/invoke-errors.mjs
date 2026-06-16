import { exitCodes } from "../config.mjs";
import { fixture } from "../fixtures.mjs";
import { runCli, validateSchema } from "../harness.mjs";
import {
  expectExit,
  expectNoJsonPayloadInStderr,
  expectProtocolFailure,
  expectStderrIncludes,
  parseJson
} from "../assertions.mjs";

export function createInvokeFailureTasks() {
  const normal = fixture("normal.md");

  const missingRef = {
    protocol_version: "0.1",
    request_id: "smoke-missing-ref",
    operation: "read",
    document: { path: normal },
    arguments: {
      limit_chars: 6000,
      page: 1
    }
  };

  const wrongType = {
    protocol_version: "0.1",
    request_id: "smoke-wrong-type",
    operation: "outline",
    document: { path: normal },
    arguments: {
      limit_chars: "many",
      page: 1
    }
  };

  const unknownOperation = {
    protocol_version: "0.1",
    request_id: "smoke-unknown-operation",
    operation: "bogus",
    document: { path: normal },
    arguments: {}
  };

  const cases = [
    {
      id: "malformed-json",
      name: "invoke malformed JSON",
      stdin: "{ \"protocol_version\":",
      stdinSummary: "malformed JSON request",
      operation: null,
      stderr: "invalid request JSON"
    },
    {
      id: "missing-read-ref",
      name: "invoke missing read ref",
      stdin: JSON.stringify(missingRef),
      stdinSummary: "schema-invalid read request without ref",
      operation: "read",
      stderr: "request schema validation failed"
    },
    {
      id: "wrong-argument-type",
      name: "invoke wrong argument type",
      stdin: JSON.stringify(wrongType),
      stdinSummary: "schema-invalid outline request with nonnumeric limit_chars",
      operation: "outline",
      stderr: "request schema validation failed"
    },
    {
      id: "unknown-operation",
      name: "invoke unknown operation",
      stdin: JSON.stringify(unknownOperation),
      stdinSummary: "schema-invalid request with unknown operation",
      operation: null,
      stderr: "request schema validation failed"
    }
  ];

  return cases.map((item) => ({
    id: `markdown-invoke-failure-${item.id}`,
    run: async () => {
      const record = await runCli(item.name, ["invoke"], {
        stdin: item.stdin,
        stdinSummary: item.stdinSummary
      });
      expectExit(record, exitCodes.input);
      expectStderrIncludes(record, item.stderr);
      expectNoJsonPayloadInStderr(record);
      const json = parseJson(record);
      validateSchema(record, "protocolResponse", json);
      expectProtocolFailure(record, json, item.operation, "INVALID_REQUEST");
    }
  }));
}
