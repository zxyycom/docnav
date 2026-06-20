export function writeJson(value: unknown) {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export function createFailureResponse(
  requestId: unknown,
  operation: unknown,
  code: string,
  details: Record<string, unknown>
) {
  return {
    protocol_version: "0.1",
    request_id: requestId,
    operation,
    ok: false,
    error: {
      code,
      message: code,
      details
    }
  };
}
