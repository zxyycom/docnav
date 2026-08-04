import { join } from "node:path";

import { writeQualityJsonArtifact } from "../../output/artifacts.ts";
import { scanJscpdAreasWithCache } from "../scanners/jscpd/area-scans.ts";
import { isToolAvailable } from "../metrics.ts";
import type { ScanContext } from "./scan-context.ts";
import type { CodeAreaFileMap } from "../../model/schema.ts";

export async function runJscpdScan(context: ScanContext, fileMap: CodeAreaFileMap): Promise<void> {
  const { metrics, toolResults, changedFiles, rawDir, root, cacheRootDir, config, fatalIssues } = context;
  if (!isToolAvailable(toolResults, "jscpd")) {
    const availability = toolResults.find((tool) => tool.name === "jscpd");
    const status = !availability || availability.reason === "tool-unavailable" ? "unavailable" : "error";
    metrics.duplicateCodeMeasurement = {
      status
    };
    console.log(status === "unavailable"
      ? "  jscpd unavailable; duplicate detection not measured"
      : "  jscpd availability check failed; duplicate detection failed");
    return;
  }

  console.log("Running jscpd...");

  const fatalIssueCount = fatalIssues.length;
  const allFragments = await scanJscpdAreasWithCache({
    cacheRootDir,
    changedFiles,
    commitSha: metrics.metadata.commitSha,
    config,
    cwd: root,
    failOnSkipped: false,
    fatalIssues,
    fileMap,
    fingerprints: context.fingerprints,
    logPrefix: "  ",
    scanKind: "current",
    toolResults
  });

  metrics.duplicateCode = allFragments;
  metrics.duplicateCodeMeasurement = {
    status: fatalIssues.length === fatalIssueCount ? "measured" : "error"
  };

  console.log(metrics.duplicateCodeMeasurement.status === "measured"
    ? `  jscpd total: ${allFragments.length} duplicate fragments`
    : `  jscpd measurement failed; ${allFragments.length} partial fragments retained`);

  writeQualityJsonArtifact(join(rawDir, "jscpd-fragments.json"), metrics.duplicateCode);
}
