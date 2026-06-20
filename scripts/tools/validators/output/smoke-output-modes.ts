import { assert } from "../assertions.ts";
import { DOCUMENT_OUTPUT_MODES, OUTPUT_MODE_CONSISTENCY } from "../config.ts";
import { readText } from "../document/markdown-docs.ts";

export function validateSmokeOutputModeCoverage(): void {
  for (const relPath of OUTPUT_MODE_CONSISTENCY.smokeMatrices) {
    assertIncludesDocumentOutputModes(relPath);
  }
}

function assertIncludesDocumentOutputModes(relPath: string): void {
  const text = readText(relPath);
  for (const mode of DOCUMENT_OUTPUT_MODES) {
    assert(
      text.includes(mode),
      `${relPath} must mention document output mode ${mode}`
    );
  }
}
