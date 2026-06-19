import { validateOutputModeConsistency } from "../output/modes.ts";
import { validateProtocolExampleSemantics } from "../protocol/examples.ts";

export function validateExampleConsistency() {
  validateProtocolExampleSemantics();
  validateOutputModeConsistency();
}
