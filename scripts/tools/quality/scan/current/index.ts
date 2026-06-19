/**
 * Current revision quality scan runner.
 */

import { buildAggregates } from "../../aggregate.ts";
import { runCpdScan } from "./cpd.ts";
import { runLizardScan } from "./lizard.ts";
import { runSccScan } from "./scc.ts";
import type { ScanContext } from "./types.ts";
import type { CodeAreaFileMap } from "../../schema.ts";

export async function scanCurrentRevision({
  context,
  scanFiles,
  fileMap
}: {
  context: ScanContext;
  fileMap: CodeAreaFileMap;
  scanFiles: string[];
}): Promise<void> {
  runSccScan(context, scanFiles);
  runLizardScan(context, scanFiles);
  await runCpdScan(context, fileMap);

  context.metrics.aggregates = buildAggregates({
    fileMetrics: context.metrics.fileMetrics,
    functionMetrics: context.metrics.functionMetrics,
    duplicateCode: context.metrics.duplicateCode,
    byLanguage: context.metrics.aggregates.byLanguage,
    config: context.config
  });
}
