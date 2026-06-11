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

export function testInvokeFailures() {
  const normal = fixture("normal.md");
  const malformed = runCli("invoke malformed JSON", ["invoke"], {
    stdin: "{ \"protocol_version\":",
    stdinSummary: "malformed JSON request"
  });
  expectExit(malformed, exitCodes.input);
  expectStderrIncludes(malformed, "invalid request JSON");
  expectNoJsonPayloadInStderr(malformed);
  const malformedJson = parseJson(malformed);
  validateSchema(malformed, "protocolResponse", malformedJson);
  expectProtocolFailure(malformed, malformedJson, null, "INVALID_REQUEST");

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
  const missingRefRecord = runCli("invoke missing read ref", ["invoke"], {
    stdin: JSON.stringify(missingRef),
    stdinSummary: "schema-invalid read request without ref"
  });
  expectExit(missingRefRecord, exitCodes.input);
  expectStderrIncludes(missingRefRecord, "request schema validation failed");
  expectNoJsonPayloadInStderr(missingRefRecord);
  const missingRefJson = parseJson(missingRefRecord);
  validateSchema(missingRefRecord, "protocolResponse", missingRefJson);
  expectProtocolFailure(missingRefRecord, missingRefJson, "read", "INVALID_REQUEST");

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
  const wrongTypeRecord = runCli("invoke wrong argument type", ["invoke"], {
    stdin: JSON.stringify(wrongType),
    stdinSummary: "schema-invalid outline request with nonnumeric limit_chars"
  });
  expectExit(wrongTypeRecord, exitCodes.input);
  expectStderrIncludes(wrongTypeRecord, "request schema validation failed");
  expectNoJsonPayloadInStderr(wrongTypeRecord);
  const wrongTypeJson = parseJson(wrongTypeRecord);
  validateSchema(wrongTypeRecord, "protocolResponse", wrongTypeJson);
  expectProtocolFailure(wrongTypeRecord, wrongTypeJson, "outline", "INVALID_REQUEST");

  const unknownOperation = {
    protocol_version: "0.1",
    request_id: "smoke-unknown-operation",
    operation: "bogus",
    document: { path: normal },
    arguments: {}
  };
  const unknownOperationRecord = runCli("invoke unknown operation", ["invoke"], {
    stdin: JSON.stringify(unknownOperation),
    stdinSummary: "schema-invalid request with unknown operation"
  });
  expectExit(unknownOperationRecord, exitCodes.input);
  expectStderrIncludes(unknownOperationRecord, "request schema validation failed");
  expectNoJsonPayloadInStderr(unknownOperationRecord);
  const unknownOperationJson = parseJson(unknownOperationRecord);
  validateSchema(unknownOperationRecord, "protocolResponse", unknownOperationJson);
  expectProtocolFailure(unknownOperationRecord, unknownOperationJson, null, "INVALID_REQUEST");
}
