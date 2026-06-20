/**
 * Markdown summary 报告生成器。
 *
 * 从 QualityMetrics 生成人类可读的 Markdown summary report。
 * 默认栏目：仓库体量/语言占比、文件排名、文件 decision tokens、函数圈复杂度、
 * 函数行数/参数数量、重复代码、watchlist、changed files 和 warnings。
 */

import { DEFAULT_CONFIG } from "../../model/config.ts";
import {
  title,
  scanInfo,
  repositorySize,
  comparisonInfo,
  footer
} from "./summary.ts";
import {
  fileRankings,
  fileDecisionTokenRankings,
  functionComplexityRankings,
  functionSizeRankings
} from "./rankings.ts";
import {
  duplicateCodeSection,
  changedFilesSection,
  warningsSection
} from "./findings.ts";
import type { QualityMetrics } from "../../model/schema.ts";

export function generateMarkdownReport(
  metrics: QualityMetrics,
  topN = 10,
  options: { timeZone?: string } = {}
): string {
  const reportOptions = {
    timeZone: options.timeZone || DEFAULT_CONFIG.report.timeZone
  };

  return [
    title(),
    scanInfo(metrics, reportOptions),
    repositorySize(metrics),
    comparisonInfo(metrics),
    fileRankings(metrics, topN),
    fileDecisionTokenRankings(metrics, topN),
    functionComplexityRankings(metrics, topN),
    functionSizeRankings(metrics, topN),
    duplicateCodeSection(metrics),
    changedFilesSection(metrics, Math.min(topN, 10)),
    warningsSection(metrics),
    footer(metrics, reportOptions)
  ].join("\n\n");
}
