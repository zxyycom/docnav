import { DOCUMENT_OUTPUT_MODES } from "../config.ts";
import { validateReadableConformanceFixtures } from "./readable-conformance-fixtures.ts";
import { validateRustOutputModes } from "./rust-output-modes.ts";
import { validateSmokeOutputModeCoverage } from "./smoke-output-modes.ts";

export function validateOutputModeConsistency(): void {
  validateRustOutputModes();
  validateSmokeOutputModeCoverage();
  validateReadableConformanceFixtures();
  console.log(
    `document output mode consistency ok: ${DOCUMENT_OUTPUT_MODES.join(", ")}`
  );
}
