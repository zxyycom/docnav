import { validateOutputModeConsistency } from "./output-modes.mjs";
import { validateProtocolExampleSemantics } from "./protocol-examples.mjs";

export function validateExampleConsistency() {
  validateProtocolExampleSemantics();
  validateOutputModeConsistency();
}
