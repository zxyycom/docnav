import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import {
  parseWarningsNdjson,
  renderGithubAnnotations
} from "../../scripts/quality/annotate-warnings.mjs";

describe("CI quality annotations", () => {
  it("renders warnings.ndjson records as non-blocking GitHub annotations", () => {
    const content = [
      JSON.stringify({
        level: "warning",
        ruleId: "scc-file-lines",
        sourceTool: "scc",
        path: "crates/docnav/src/lib.rs",
        line: 12,
        codeArea: "rust-production",
        metric: "lines",
        value: 500,
        comparisonBasis: "delta",
        baselineValue: 350,
        deltaValue: 150,
        message: "Large file",
        suggestion: "Split it"
      })
    ].join("\n");

    const parsed = parseWarningsNdjson(content);
    const annotations = renderGithubAnnotations(parsed.warnings);

    assert.deepEqual(parsed.diagnostics, []);
    assert.equal(annotations.length, 1);
    assert.match(annotations[0], /^::warning /);
    assert.match(annotations[0], /file=crates\/docnav\/src\/lib.rs/);
    assert.match(annotations[0], /line=12/);
    assert.match(annotations[0], /baseline=350/);
    assert.match(annotations[0], /delta=150/);
  });

  it("does not render info watchlist records as GitHub warning annotations", () => {
    const annotations = renderGithubAnnotations([
      {
        level: "info",
        ruleId: "fixture-watchlist",
        path: "test/fixtures/sample.rs",
        line: null,
        message: "Watchlist only"
      }
    ]);

    assert.deepEqual(annotations, []);
  });
});
