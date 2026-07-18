import { FIELDS, OPERATIONS } from "../config.ts";
import { validateErrorDetails } from "./examples/error-details.ts";
import { validateManifestSemantics } from "./examples/manifest.ts";
import { validateExampleBudget } from "./examples/budget.ts";
import { jsonObject } from "./examples/json.ts";
import { validateProtocolPair } from "./examples/protocol-pairs.ts";

export function validateProtocolExampleSemantics() {
  validateProtocolOperationExamples();
  validateErrorDetails();
  validateManifestSemantics();
}

function validateProtocolOperationExamples(): void {
  for (const operation of OPERATIONS) {
    const { request, response } = validateProtocolPair(operation);
    const result = jsonObject(response[FIELDS.result], `${operation} response result`);
    validateExampleBudget(operation, request, result);
  }

  console.log(`protocol examples ok: ${OPERATIONS.length} operation(s)`);
}
