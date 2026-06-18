import { formatTable } from "./table.ts";

export function title() {
  const nonBlocking = "**⚠️ 非阻断观测快照 — Lizard、scc 和 PMD CPD 指标值不作为合并阻断条件。Clippy 继续承担 Rust 阻断式 lint gate。**";
  return [
    "# Docnav Code Quality Snapshot",
    "",
    nonBlocking
  ].join("\n");
}

export function scanInfo(metrics: ExternalValue, options: ExternalValue) {
  const m = metrics.metadata;
  const tools = m.tools.map((tool: ExternalValue) => `- **${tool.name}**: ${tool.version} (via ${tool.source})`).join("\n");
  const timestamp = formatReportTimestamp(m.timestamp, options?.timeZone);
  return [
    "## 扫描信息",
    "",
    `- **Schema version**: ${m.schemaVersion}`,
    `- **Timestamp**: ${timestamp}`,
    `- **Commit**: \`${m.commitSha}\``,
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

export function comparisonInfo(metrics: ExternalValue) {
  if (metrics.comparisonStatus === "input-unchanged") {
    return [
      "## Comparison",
      "",
      "**代码输入未变化。** 本次变更未修改任何纳入扫描的代码文件。",
      "当前快照已生成，但不生成动态复杂度或重复代码 annotation。"
    ].join("\n");
  }

  if (metrics.comparisonStatus === "baseline-unavailable") {
    const reason = baselineUnavailableReason(metrics.baseline.status);
    return [
      "## Comparison",
      "",
      `**⚠️ Baseline 不可用:** ${reason} (\`${metrics.baseline.status}\`)。`,
      "趋势比较无法生成，报告仅展示当前快照。"
    ].join("\n");
  }

  if (metrics.comparisonStatus === "compared" && metrics.baseline.metadata) {
    const baseline = metrics.baseline;
    return [
      "## Comparison",
      "",
      `- **Baseline commit**: \`${baseline.commitSha}\``,
      `- **Baseline date**: ${baseline.commitDate || "ExternalValue"}`,
      `- **Selection reason**: ${baseline.metadata.selectionReason}`,
      "",
      "### Code Area 指纹对比",
      "",
      fingerprintTable(metrics)
    ].join("\n");
  }

  return "";
}

export function repositorySize(metrics: ExternalValue) {
  const agg = metrics.aggregates;
  const lines: ExternalValue[] = [];

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

export function footer(metrics: ExternalValue, options: ExternalValue) {
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

function baselineUnavailableReason(status: ExternalValue) {
  if (status === "baseline-skipped") return "Baseline scan was skipped";
  if (status === "history-unavailable") return "Git history 不足";
  if (status === "no-baseline-commit") return "找不到 previous-code baseline commit";
  if (status === "baseline-materialization-failed") return "Baseline commit 导出失败";
  if (status === "baseline-scan-failed") return "Baseline 扫描失败";
  return "未知原因";
}

export function formatReportTimestamp(timestamp: ExternalValue, timeZone: ExternalValue) {
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

function fingerprintTable(metrics: ExternalValue) {
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

function appendLanguageTable(lines: ExternalValue, agg: ExternalValue) {
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

function appendCodeAreaTable(lines: ExternalValue, agg: ExternalValue) {
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
