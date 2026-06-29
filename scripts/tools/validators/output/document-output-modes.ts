import { DOCUMENT_OUTPUT_MODES } from "../config.ts";
import { validateReadableConformanceFixtures } from "./readable-conformance-fixtures.ts";

export function validateOutputModeConsistency(): void {
  validateReadableConformanceFixtures();
  console.log(
    `document output mode consistency ok: ${DOCUMENT_OUTPUT_MODES.join(", ")}`
  );
}
