import { parseNdjson } from "../../tools/foundation/src/ndjson.ts";
import { isRecord } from "../../tools/foundation/src/type-guards.ts";

export type RenderableWarning = {
  baselineValue?: number | null;
  comparisonBasis?: string;
  deltaValue?: number | null;
  level?: string;
  line?: number | null;
  message: string;
  path: string;
  ruleId: string;
  suggestion?: string;
};

export function parseWarningsNdjson(content: string): { diagnostics: string[]; warnings: RenderableWarning[] } {
  const warnings: RenderableWarning[] = [];
  const diagnostics: string[] = [];
  const parsed = parseNdjson(content);

  for (const diagnostic of parsed.diagnostics) {
    diagnostics.push(`line ${diagnostic.line}: ${diagnostic.message}`);
  }

  for (const record of parsed.records) {
    if (isRenderableWarning(record.value)) {
      warnings.push(record.value);
    } else {
      diagnostics.push(`line ${record.line}: missing required warning fields`);
    }
  }

  return { warnings, diagnostics };
}

function isRenderableWarning(record: unknown): record is RenderableWarning {
  return isRecord(record) &&
    typeof record.ruleId === "string" &&
    typeof record.path === "string" &&
    typeof record.message === "string";
}
