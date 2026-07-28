import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import type { AcceptedWarningConfig, DuplicateCodeFragment, QualityConfig } from "../../model/schema.ts";
import { generateWarningChannels } from "./generator.ts";
import { TEST_QUALITY_CONFIG } from "../../../test/config.ts";

describe("quality warning generation", () => {
  it("adds configured accepted reasons without relying on duplicate line numbers", () => {
    const warnings = generateWarningChannels({
      baseline: null,
      comparisonStatus: "baseline-unavailable",
      config: configWithAcceptedWarnings([acceptedProtocolOperationDuplicateAcceptance()]),
      duplicates: [acceptedProtocolOperationDuplicate({ startLineOffset: 20 })],
      files: [],
      functions: [],
      scope: { changed: false, changedFiles: [] }
    });

    assert.equal(warnings.all.length, 1);
    assert.match(warnings.all[0]!.acceptedReason ?? "", /separate protocol request and result boundaries/);
    assert.deepEqual(warnings.changed, []);
    assert.deepEqual(warnings.regressions, []);
  });

  it("warns when an accepted warning rule no longer matches any generated warning", () => {
    const warnings = generateWarningChannels({
      baseline: null,
      comparisonStatus: "baseline-unavailable",
      config: configWithAcceptedWarnings([
        {
          ruleId: "jscpd-duplicate-code",
          sourceTool: "jscpd",
          metric: "duplicate-tokens",
          value: 999,
          reason: "stale acceptance for test"
        }
      ]),
      duplicates: [acceptedProtocolOperationDuplicate()],
      files: [],
      functions: [],
      scope: { changed: false, changedFiles: [] },
      validateAcceptedWarnings: true
    });

    const unmatched = warnings.all.find((warning) => warning.ruleId === "quality-accepted-warning-unmatched");

    assert.ok(unmatched);
    assert.match(unmatched.message, /value=999/);
    assert.equal(unmatched.acceptedReason, undefined);
  });
});

function acceptedProtocolOperationDuplicate({
  startLineOffset = 0
}: {
  startLineOffset?: number;
} = {}): DuplicateCodeFragment {
  return {
    id: 1,
    tokenCount: 86,
    lineCount: 14,
    hitsChangedScope: false,
    codeAreas: ["rust-production"],
    locations: [
      {
        path: "crates/shared/protocol/src/envelope.rs",
        startLine: 62 + startLineOffset,
        endLine: 75 + startLineOffset,
        codeArea: "rust-production"
      },
      {
        path: "crates/shared/protocol/src/operation_result.rs",
        startLine: 14 + startLineOffset,
        endLine: 27 + startLineOffset,
        codeArea: "rust-production"
      }
    ]
  };
}

function acceptedProtocolOperationDuplicateAcceptance(): AcceptedWarningConfig {
  return {
    ruleId: "jscpd-duplicate-code",
    sourceTool: "jscpd",
    codeArea: "rust-production",
    metric: "duplicate-tokens",
    suggestionIncludes: [
      "crates/shared/protocol/src/envelope.rs",
      "crates/shared/protocol/src/operation_result.rs"
    ],
    reason:
      "OperationArguments::operation and OperationResult::operation map the same Operation enum variants at separate protocol request and result boundaries."
  };
}

function configWithAcceptedWarnings(acceptedWarnings: AcceptedWarningConfig[]): QualityConfig {
  return {
    ...TEST_QUALITY_CONFIG,
    acceptedWarnings
  };
}
