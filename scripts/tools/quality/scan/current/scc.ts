import { join } from "node:path";

import { writeQualityJsonArtifact } from "../../artifacts.ts";
import { scanWithScc } from "../../tools/scc.ts";
import { isToolAvailable, normalizeFileMetrics } from "../metrics.ts";
import type { ScanContext } from "./types.ts";
import type { CodeAreaAggregate, FileMetric, QualityConfig } from "../../schema.ts";

export function runSccScan(context: ScanContext, scanFiles: string[]): void {
  const { metrics, toolResults, rawDir, fatalIssues, root, config } = context;
  if (!isToolAvailable(toolResults, "scc")) return;

  console.log("Running scc...");

  const sccResult = scanWithScc({
    cwd: root,
    includePaths: scanFiles,
    excludeDirs: config.excludeDirs,
    toolConfig: config.tools.scc
  });

  if (sccResult.ok) {
    metrics.fileMetrics = normalizeFileMetrics(sccResult.files ?? [], {
      changedFiles: context.changedFiles,
      config
    });
    metrics.aggregates.byLanguage = sccResult.aggregates?.byLanguage ?? [];
    console.log(`  scc: ${metrics.fileMetrics.length} files, ${metrics.aggregates.byLanguage.length} languages`);
  } else {
    fatalIssues.push({ tool: "scc", phase: "current-scan", error: sccResult.error });
    console.log(`  ❌ scc execution/config/schema error: ${sccResult.error}`);
  }

  metrics.aggregates.byCodeArea = buildFileAreaAggregates(metrics.fileMetrics, config);
  metrics.aggregates.overall = {
    totalFiles: metrics.fileMetrics.length,
    totalLines: metrics.fileMetrics.reduce((s, f) => s + f.lines, 0),
    totalCodeLines: metrics.fileMetrics.reduce((s, f) => s + (f.codeLines || 0), 0),
    totalFunctions: 0
  };

  writeQualityJsonArtifact(join(rawDir, "scc-output.json"), metrics.fileMetrics);
}

function buildFileAreaAggregates(fileMetrics: FileMetric[], config: QualityConfig): CodeAreaAggregate[] {
  const areaAggMap = new Map<string, CodeAreaAggregate>();

  for (const file of fileMetrics) {
    const existing = areaAggMap.get(file.codeArea);
    if (existing) {
      existing.files++;
      existing.lines += file.lines;
    } else {
      const areaDef = config.codeAreas[file.codeArea];
      areaAggMap.set(file.codeArea, {
        codeArea: file.codeArea,
        files: 1,
        lines: file.lines,
        functions: 0,
        warningPolicy: areaDef?.warningPolicy || "moderate"
      });
    }
  }

  return Array.from(areaAggMap.values()).sort((a, b) => b.lines - a.lines);
}
