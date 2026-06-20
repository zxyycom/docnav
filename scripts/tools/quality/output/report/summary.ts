import { formatTable } from "./markdown-table.ts";
import type { AggregateMetrics, BaselineStatus, QualityMetrics } from "../../model/schema.ts";

export function title() {
  const nonBlocking = "**⚠️ 非阻断观测快照 — Lizard、scc 和 PMD CPD 指标值不作为合并阻断条件。Clippy 继续承担 Rust 阻断式 lint gate。**";
  return [
    "# Docnav Code Quality Snapshot",
    "",
    nonBlocking
  ].join("\n");
}

type ReportOptions = {
  timeZone: string;
};

export function scanInfo(metrics: QualityMetrics, options: ReportOptions): string {
  const m = metrics.metadata;
  const tools = m.tools.map((tool) => `- **${tool.name}**: ${tool.version} (via ${tool.source})`).join("\n");
  const timestamp = formatReportTimestamp(m.timestamp, options?.timeZone);
  return [
    "## 扫描信息",
    "",
    `- **Schema version**: ${m.schemaVersion}`,
    `- **Timestamp**: ${timestamp}`,
    `- **Commit**: ${formatCommitDisplay(m.commitSha, m.commitTitle)}`,
    `- **Config version**: ${m.configVersion}`,
    `- **Scope**: ${m.scope.include.join(", ")}`,
    "",
    "### 工具",
    tools,
    "",
    `- **Baseline status**: \`${metrics.baseline.status}\``,
    `- **Comparison status**: \`${metrics.comparisonStatus}\``
  ].join("\n");
}

export function comparisonInfo(metrics: QualityMetrics): string {
  if (metrics.comparisonStatus === "input-unchanged") {
    return inputUnchangedComparisonSection();
  }

  if (metrics.comparisonStatus === "baseline-unavailable") {
    return baselineUnavailableComparisonSection(metrics.baseline.status);
  }

  if (metrics.comparisonStatus === "compared" && metrics.baseline.metadata) {
    return comparedComparisonSection(metrics);
  }

  return "";
}

function inputUnchangedComparisonSection(): string {
  return [
    "## Comparison",
    "",
    "**代码输入未变化。** 本次变更未修改任何纳入扫描的代码文件。",
    "当前快照已生成，但不生成动态复杂度或重复代码 annotation。"
  ].join("\n");
}

function baselineUnavailableComparisonSection(status: BaselineStatus | string): string {
  const reason = baselineUnavailableReason(status);
  return [
    "## Comparison",
    "",
    `**⚠️ Baseline 不可用:** ${reason} (\`${status}\`)。`,
    "Baseline delta 不可用，报告仅展示当前快照。"
  ].join("\n");
}

function comparedComparisonSection(metrics: QualityMetrics): string {
  const baseline = metrics.baseline;
  const baselineMetadata = metrics.baseline.metadata;
  if (!baselineMetadata) {
    return "";
  }

  return [
    "## Comparison",
    "",
    `- **Baseline commit**: ${formatCommitDisplay(baseline.commitSha || "unknown", baselineMetadata.commitTitle)}`,
    `- **Baseline date**: ${baseline.commitDate || "unknown"}`,
    `- **Selection reason**: ${baselineMetadata.selectionReason}`,
    "",
    "### Code Area 指纹对比",
    "",
    fingerprintTable(metrics)
  ].join("\n");
}

export function repositorySize(metrics: QualityMetrics): string {
  const agg = metrics.aggregates;
  const lines: string[] = [];

  lines.push("## 仓库体量与语言占比");
  lines.push("");

  if (agg.overall.totalFiles > 0) {
    lines.push(`- **Total files**: ${agg.overall.totalFiles}`);
    lines.push(`- **Total lines**: ${agg.overall.totalLines.toLocaleString()}`);
    lines.push(`- **Total code lines**: ${agg.overall.totalCodeLines.toLocaleString()}`);
    lines.push(`- **Total functions**: ${agg.overall.totalFunctions}`);
  } else {
    lines.push("*(no file metrics available)*");
  }

  lines.push("");

  appendLanguageTable(lines, agg);
  appendCodeAreaTable(lines, agg);

  return lines.join("\n");
}

export function footer(metrics: QualityMetrics, options: ReportOptions): string {
  const timestamp = formatReportTimestamp(metrics.metadata.timestamp, options?.timeZone);
  return [
    "---",
    "",
    `*Report generated at ${timestamp} by Docnav Code Quality Observability*`,
    "",
    `*Config version: ${metrics.metadata.configVersion} | Schema version: ${metrics.metadata.schemaVersion}*`,
    "",
    "*⚠️ 本报告为**非阻断观测快照**。Lizard、scc 和 PMD CPD 指标值不作为合并阻断条件。Clippy 继续承担 Rust 阻断式 lint gate。*"
  ].join("\n");
}

function baselineUnavailableReason(status: BaselineStatus | string): string {
  if (status === "baseline-skipped") return "Baseline scan was skipped";
  if (status === "history-unavailable") return "Git history 不足";
  if (status === "no-baseline-commit") return "找不到 previous-code baseline commit";
  if (status === "baseline-materialization-failed") return "Baseline commit 导出失败";
  if (status === "baseline-scan-failed") return "Baseline 扫描失败";
  return "未知原因";
}

function formatCommitDisplay(sha: string, title: string | null | undefined): string {
  return title ? `\`${sha}\` - ${title}` : `\`${sha}\``;
}

export function formatReportTimestamp(timestamp: string, timeZone: string): string {
  if (typeof timestamp !== "string" || timestamp.length === 0) {
    throw new Error("report timestamp is required");
  }
  if (typeof timeZone !== "string" || timeZone.length === 0) {
    throw new Error("report timeZone is required");
  }

  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) {
    throw new Error(`invalid report timestamp: ${timestamp}`);
  }

  const formatter = new Intl.DateTimeFormat("en-GB", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hourCycle: "h23",
    timeZoneName: "longOffset"
  });
  const parts = Object.fromEntries(formatter.formatToParts(date).map((part) => [part.type, part.value]));

  return [
    `${parts.year}-${parts.month}-${parts.day}`,
    `${parts.hour}:${parts.minute}:${parts.second}`,
    `${parts.timeZoneName} (${timeZone}; source ${timestamp})`
  ].join(" ");
}

function fingerprintTable(metrics: QualityMetrics): string {
  const rows = [["Code Area", "Current Files", "Baseline Files", "Match"]];
  const current = metrics.currentFingerprints || {};
  const baseline = metrics.baselineFingerprints || {};

  for (const area of Object.keys({ ...current, ...baseline })) {
    const currentFingerprint = current[area];
    const baselineFingerprint = baseline[area];
    const currentCount = currentFingerprint?.fileCount ?? 0;
    const baselineCount = baselineFingerprint?.fileCount ?? 0;
    const match = currentFingerprint?.fingerprint === baselineFingerprint?.fingerprint ? "✓" : "✗ changed";

    rows.push([area, String(currentCount), String(baselineCount), match]);
  }

  return formatTable(rows);
}

function appendLanguageTable(lines: string[], agg: AggregateMetrics): void {
  if (agg.byLanguage.length === 0) return;

  lines.push("### By Language");
  lines.push("");
  const rows = [["Language", "Files", "Lines", "Code", "Comments", "Blanks"]];
  for (const lang of agg.byLanguage) {
    rows.push([
      lang.language,
      String(lang.files),
      lang.lines.toLocaleString(),
      lang.codeLines.toLocaleString(),
      lang.commentLines.toLocaleString(),
      lang.blankLines.toLocaleString()
    ]);
  }
  lines.push(formatTable(rows));
}

function appendCodeAreaTable(lines: string[], agg: AggregateMetrics): void {
  if (agg.byCodeArea.length === 0) return;

  lines.push("");
  lines.push("### By Code Area");
  lines.push("");
  const rows = [["Code Area", "Files", "Lines", "Functions", "Policy"]];
  for (const area of agg.byCodeArea) {
    rows.push([
      area.codeArea,
      String(area.files),
      area.lines.toLocaleString(),
      String(area.functions || 0),
      area.warningPolicy
    ]);
  }
  lines.push(formatTable(rows));
}
