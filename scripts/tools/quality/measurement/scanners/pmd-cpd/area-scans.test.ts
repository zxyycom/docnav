import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import { planPmdCpdAreaScanTasks } from "./area-scans.ts";

// @case AUX-QUALITY-CPD-TASK-001
describe("PMD CPD tasks", () => {
  it("plans one scan task per code area", () => {
    const tasks = planPmdCpdAreaScanTasks([
      {
        area: "rust-production",
        files: ["crates/b/src/lib.rs", "crates/a/src/lib.rs"],
        minimumTokens: 75
      },
      {
        area: "node-production-scripts",
        files: ["scripts/a.ts", "scripts/b.ts"],
        minimumTokens: 75
      }
    ]);

    assert.deepEqual(tasks.map((task) => task.id), [
      "pmd-cpd:rust-production",
      "pmd-cpd:node-production-scripts"
    ]);
    assert.deepEqual(tasks[0]!.files, [
      "crates/a/src/lib.rs",
      "crates/b/src/lib.rs"
    ]);
  });
});
