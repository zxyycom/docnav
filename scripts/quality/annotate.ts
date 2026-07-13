#!/usr/bin/env bun

/**
 * Render unaccepted quality warning records as non-blocking GitHub Actions annotations.
 *
 * Input is the full warnings-all.ndjson produced by scripts/quality/scan.ts.
 * This script never fails the job for metric values; malformed warning records
 * are skipped with diagnostics so the artifact remains the source of truth.
 */

import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

import { errorMessage } from "../tools/foundation/src/errors.ts";
import { renderGithubAnnotations } from "./annotate/github.ts";
import { parseWarningsNdjson, selectAnnotationWarnings } from "./annotate/warnings.ts";

export { renderGithubAnnotations } from "./annotate/github.ts";
export { parseWarningsNdjson } from "./annotate/warnings.ts";
export type { RenderableWarning } from "./annotate/warnings.ts";

function main() {
  const warningsPath = process.argv[2] || "artifacts/docnav-quality/warnings-all.ndjson";
  const limit = parseAnnotationLimit(process.argv[3] ?? "5");
  try {
    const content = readFileSync(warningsPath, "utf8");
    const { warnings, diagnostics } = parseWarningsNdjson(content);
    for (const diagnostic of diagnostics) {
      console.log(`Quality warning annotation skipped: ${diagnostic}`);
    }
    const renderedWarnings = selectAnnotationWarnings(warnings);
    for (const annotation of renderGithubAnnotations(renderedWarnings.slice(0, limit))) {
      console.log(annotation);
    }
    if (renderedWarnings.length > limit) {
      console.log(`Quality warning annotation limit: showing ${limit} of ${renderedWarnings.length}; see ${warningsPath}`);
    }
  } catch (err: unknown) {
    console.log(`No quality warnings rendered: ${errorMessage(err)}`);
    return;
  }
}

function parseAnnotationLimit(value: string): number {
  const limit = Number.parseInt(value, 10);
  if (!Number.isInteger(limit) || limit <= 0 || String(limit) !== value) {
    throw new Error(`quality annotation limit must be a positive integer: ${value}`);
  }
  return limit;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main();
}
