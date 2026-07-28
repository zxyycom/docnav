import assert from "node:assert/strict";
import test from "node:test";

import { exitCodeForDiagnostics } from "./cli.ts";
import { diagnostic } from "./model.ts";

test("uses distinct exit statuses for discovery, runner, Case, and query failures", () => {
  assert.equal(exitCodeForDiagnostics([
    diagnostic("unsupported-entity-shape", "static", "unsupported")
  ]), 3);
  assert.equal(exitCodeForDiagnostics([
    diagnostic("runner-report-failed", "runner", "failed")
  ]), 4);
  assert.equal(exitCodeForDiagnostics([
    diagnostic("entity.case-missing", "case", "missing")
  ]), 5);
  assert.equal(exitCodeForDiagnostics([
    diagnostic("query.case-not-found", "query", "unknown")
  ]), 6);
});
