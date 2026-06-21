import { join } from "node:path";

import { writeQualityJsonArtifact } from "../../output/artifacts.ts";
import { scanPmdCpdAreasWithCache } from "../scanners/pmd-cpd/area-scans.ts";
import { isToolAvailable } from "../metrics.ts";
import type { ScanContext } from "./scan-context.ts";
import type { CodeAreaFileMap } from "../../model/schema.ts";

export async function runPmdCpdScan(context: ScanContext, fileMap: CodeAreaFileMap): Promise<void> {
  const { metrics, toolResults, changedFiles, rawDir, root, config } = context;
  if (!isToolAvailable(toolResults, "pmd-cpd")) {
    console.log("  CPD not available, skipping duplicate detection");
    return;
  }

  console.log("Running PMD CPD...");

  const allFragments = await scanPmdCpdAreasWithCache({
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

  console.log(`  CPD total: ${allFragments.length} duplicate fragments`);

  writeQualityJsonArtifact(join(rawDir, "cpd-fragments.json"), metrics.duplicateCode);
}
