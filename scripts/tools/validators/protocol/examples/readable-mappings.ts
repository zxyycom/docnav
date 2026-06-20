import { assertDeepEqual } from "../../assertions.ts";
import {
  FIELDS,
  MCP_EXAMPLE_FILE,
  OPERATIONS,
  READABLE_EXAMPLE_FILE,
} from "../../config.ts";
import { readJson } from "../../json/files.ts";
import { validateExampleBudget } from "./budget.ts";
import { jsonObject } from "./json.ts";
import { validateProtocolPair } from "./protocol-pairs.ts";

function toReadablePayload(_operation: string, protocolResult: unknown): unknown {
  return protocolResult;
}

export function validateProtocolReadableMappings() {
  for (const operation of OPERATIONS) {
    const { request, response } = validateProtocolPair(operation);
    const result = jsonObject(response[FIELDS.result], `${operation} response result`);
    validateExampleBudget(operation, request, result);

    const readable = readJson(READABLE_EXAMPLE_FILE.result(operation));
    assertDeepEqual(
      readable,
      toReadablePayload(operation, response[FIELDS.result]),
      `${operation} readable JSON must preserve protocol result semantics`,
    );

    const mcp = jsonObject(readJson(MCP_EXAMPLE_FILE.response(operation)), `${operation} MCP response`);
    const mcpResult = jsonObject(mcp[FIELDS.result], `${operation} MCP result`);
    assertDeepEqual(
      mcpResult[FIELDS.structuredContent],
      readable,
      `${operation} MCP structuredContent must match readable JSON example`,
    );
  }

  console.log(
    `protocol/readable mapping ok: ${OPERATIONS.length} operation(s)`,
  );
}
