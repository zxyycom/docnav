import { validateErrorDetails } from "./examples/error-details.ts";
import { validateManifestSemantics } from "./examples/manifest.ts";
import { validateProtocolReadableMappings } from "./examples/readable-mappings.ts";

export function validateProtocolExampleSemantics() {
  validateProtocolReadableMappings();
  validateErrorDetails();
  validateManifestSemantics();
}
