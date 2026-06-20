#!/usr/bin/env node

/**
 * Render quality warning records as non-blocking GitHub Actions annotations.
 *
 * Input is the changed-scope warnings.ndjson produced by scripts/quality/scan.ts.
 * This script never fails the job for metric values; malformed warning records
 * are skipped with diagnostics so the artifact remains the source of truth.
 */

import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

import { parseNdjson } from "../tools/ndjson.ts";
import { errorMessage } from "../tools/errors.ts";
import { isRecord } from "../tools/type-guards.ts";

type RenderableWarning = {
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

export function renderGithubAnnotations(warnings: RenderableWarning[]): string[] {
  return warnings.filter((warning) => warning.level !== "info").map((warning) => {
    const attrs = [
      ["file", warning.path],
      ["line", warning.line],
      ["title", warning.ruleId]
    ]
      .filter(([, value]) => value !== null && value !== undefined && value !== "")
      .map(([key, value]) => `${key}=${escapeProperty(String(value))}`)
      .join(",");

    const message = [
      warning.message,
      warning.comparisonBasis ? `basis=${warning.comparisonBasis}` : null,
      warning.baselineValue !== null && warning.baselineValue !== undefined
        ? `baseline=${warning.baselineValue}`
        : null,
      warning.deltaValue !== null && warning.deltaValue !== undefined
        ? `delta=${warning.deltaValue}`
        : null,
      warning.suggestion || null
    ]
      .filter(Boolean)
      .join(" | ");

    return `::warning ${attrs}::${escapeData(message)}`;
  });
}

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

function escapeData(value: string): string {
  return value
    .replace(/%/g, "%25")
    .replace(/\r/g, "%0D")
    .replace(/\n/g, "%0A");
}

function escapeProperty(value: string): string {
  return escapeData(value)
    .replace(/:/g, "%3A")
    .replace(/,/g, "%2C");
}

function main() {
  const warningsPath = process.argv[2] || "artifacts/docnav-quality/warnings.ndjson";
  try {
    const content = readFileSync(warningsPath, "utf8");
    const { warnings, diagnostics } = parseWarningsNdjson(content);
    for (const diagnostic of diagnostics) {
      console.log(`Quality warning annotation skipped: ${diagnostic}`);
    }
    for (const annotation of renderGithubAnnotations(warnings)) {
      console.log(annotation);
    }
  } catch (err: unknown) {
    console.log(`No quality warnings rendered: ${errorMessage(err)}`);
    return;
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main();
}
