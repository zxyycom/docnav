#!/usr/bin/env node

/**
 * Render quality warning records as non-blocking GitHub Actions annotations.
 *
 * Input is warnings.ndjson produced by scripts/quality-scan.ts. This script
 * never fails the job for metric values; malformed warning records are skipped
 * with diagnostics so the artifact remains the source of truth.
 */

import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

import { errorMessage } from "./tools/types.ts";

export function renderGithubAnnotations(warnings: ExternalValue) {
  return warnings.filter((warning: ExternalValue) => warning.level !== "info").map((warning: ExternalValue) => {
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

export function parseWarningsNdjson(content: ExternalValue) {
  const warnings: ExternalValue[] = [];
  const diagnostics: ExternalValue[] = [];

  for (const [index, line] of content.split(/\r?\n/).entries()) {
    if (!line.trim()) continue;
    try {
      const record = JSON.parse(line);
      if (isRenderableWarning(record)) {
        warnings.push(record);
      } else {
        diagnostics.push(`line ${index + 1}: missing required warning fields`);
      }
    } catch (err: unknown) {
      diagnostics.push(`line ${index + 1}: invalid JSON: ${errorMessage(err)}`);
    }
  }

  return { warnings, diagnostics };
}

function isRenderableWarning(record: ExternalValue) {
  return record &&
    typeof record === "object" &&
    typeof record.ruleId === "string" &&
    typeof record.path === "string" &&
    typeof record.message === "string";
}

function escapeData(value: ExternalValue) {
  return value
    .replace(/%/g, "%25")
    .replace(/\r/g, "%0D")
    .replace(/\n/g, "%0A");
}

function escapeProperty(value: ExternalValue) {
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
