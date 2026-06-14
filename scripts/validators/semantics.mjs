import { validateMcpBridgeHandoffDocs } from "./mcp-handoff.mjs";
import { validateOutputModeConsistency } from "./output-modes.mjs";
import { validateProtocolExampleSemantics } from "./protocol-examples.mjs";

export { collectMcpBridgeHandoffDocViolations } from "./mcp-handoff.mjs";

export function validateExampleSemantics() {
  validateProtocolExampleSemantics();
  validateMcpBridgeHandoffDocs();
  validateOutputModeConsistency();
}
