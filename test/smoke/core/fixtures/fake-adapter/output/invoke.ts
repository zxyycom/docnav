import type {
  AdapterRequest,
  FakeAdapterOptions
} from "../types.ts";
import { resultFor } from "./operations.ts";
import { createFailureResponse, isRecord, writeJson } from "./response.ts";

export function writeInvoke(options: FakeAdapterOptions, value: unknown) {
  if (options.mode === "invoke-exit") {
    console.error(`${options.id} invoke failed intentionally`);
    process.exit(9);
  }
  if (options.mode === "invoke-invalid-json") {
    process.stdout.write("{ invalid json");
    return;
  }
  if (!isRecord(value)) {
    writeJson(createFailureResponse("unknown", null, "INVALID_REQUEST", {
      field: "stdin",
      reason: "missing request JSON"
    }));
    process.exit(2);
  }

  const request = value as AdapterRequest;
  writeJson(successResponse(options, request));
}

function successResponse(options: FakeAdapterOptions, request: AdapterRequest) {
  if (options.mode === "invoke-schema-invalid") {
    return {
      protocol_version: "0.1",
      request_id: request.request_id,
      operation: request.operation,
      ok: true,
      result: {
        entries: "not an array"
      }
    };
  }

  return {
    protocol_version: "0.1",
    request_id: request.request_id,
    operation: request.operation,
    ok: true,
    result: resultFor(options, request)
  };
}
