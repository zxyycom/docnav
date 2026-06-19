import { join } from "node:path";

import { writeQualityJsonArtifact } from "../../artifacts.ts";
import { scanCpdPartitionsWithCache } from "../../cpd/tasks/index.ts";
import { isToolAvailable } from "../metrics.ts";
import type { ScanContext } from "./types.ts";
import type { CodeAreaFileMap, DuplicateCodeFragment, QualityMetrics } from "../../schema.ts";

export async function runCpdScan(context: ScanContext, fileMap: CodeAreaFileMap): Promise<void> {
  const { metrics, toolResults, changedFiles, rawDir, root, config } = context;
  if (!isToolAvailable(toolResults, "pmd-cpd")) {
    console.log("  CPD not available, skipping duplicate detection");
    return;
  }

  console.log("Running PMD CPD...");

  const allFragments = await scanCpdPartitionsWithCache({
    cacheRootDir: root,
    changedFiles,
    commitSha: metrics.metadata.commitSha,
    config,
    cwd: root,
    failOnSkipped: false,
    fileMap,
    fingerprints: context.fingerprints,
    logPrefix: "  ",
    scanKind: "current",
    toolResults
  });

  metrics.duplicateCode = allFragments;
  updateDuplicateCounts(metrics, allFragments);

  console.log(`  CPD total: ${allFragments.length} duplicate fragments`);

  writeQualityJsonArtifact(join(rawDir, "cpd-fragments.json"), metrics.duplicateCode);
}

function updateDuplicateCounts(metrics: QualityMetrics, allFragments: DuplicateCodeFragment[]): void {
  const dupByArea = new Map<string, number>();
  for (const dup of allFragments) {
    for (const area of dup.codeAreas) {
      dupByArea.set(area, (dupByArea.get(area) || 0) + 1);
    }
  }
  for (const agg of metrics.aggregates.byCodeArea) {
    agg.duplicateFragments = dupByArea.get(agg.codeArea) || 0;
  }
}
