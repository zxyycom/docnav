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

import { errorMessage } from "../tools/errors.ts";
import { renderGithubAnnotations } from "./annotate/github.ts";
import { parseWarningsNdjson } from "./annotate/warnings.ts";

export { renderGithubAnnotations } from "./annotate/github.ts";
export { parseWarningsNdjson } from "./annotate/warnings.ts";
export type { RenderableWarning } from "./annotate/warnings.ts";

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
