/**
 * Markdown summary 报告生成器。
 *
 * 从 QualityMetrics 生成人类可读的 Markdown summary report。
 * 默认栏目：仓库体量/语言占比、文件排名、文件复杂度、函数圈复杂度、
 * 函数行数/参数数量、重复代码、watchlist、changed files、trend section。
 *
 * 来源：openspec/changes/implement-code-quality-observability/tasks.md task 2.4
 */

import { DEFAULT_CONFIG } from "../config.ts";
import {
  title,
  scanInfo,
  repositorySize,
  comparisonInfo,
  footer
} from "./summary.ts";
import {
  fileRankings,
  fileComplexityRankings,
  functionComplexityRankings,
  functionSizeRankings
} from "./rankings.ts";
import {
  duplicateCodeSection,
  changedFilesSection,
  trendSection,
  warningsSection
} from "./findings.ts";

/**
 * 生成 Markdown summary。
 *
 * @param {import('../schema.ts').QualityMetrics} metrics
 * @param {number} topN - Top N 数量
 * @returns {string} Markdown report
 */
export function generateMarkdownReport(metrics: any, topN = 10, options: any = {}) {
  const reportOptions = {
    timeZone: options.timeZone || DEFAULT_CONFIG.report.timeZone
  };

  return [
    title(),
    scanInfo(metrics, reportOptions),
    repositorySize(metrics),
    comparisonInfo(metrics),
    fileRankings(metrics, topN),
    fileComplexityRankings(metrics, topN),
    functionComplexityRankings(metrics, topN),
    functionSizeRankings(metrics, topN),
    duplicateCodeSection(metrics),
    changedFilesSection(metrics),
    trendSection(metrics),
    warningsSection(metrics),
    footer(metrics, reportOptions)
  ].join("\n\n");
}
