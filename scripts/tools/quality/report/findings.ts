import { formatTable } from "./table.ts";

export function duplicateCodeSection(metrics: any) {
  const lines: any[] = [];
  lines.push("## 重复代码检测");
  lines.push("");

  const duplicates = metrics.duplicateCode;
  if (duplicates.length === 0) {
    lines.push("✅ 未发现重复代码片段（在配置的 minimum tokens 阈值以上）");
    return lines.join("\n");
  }

  const byArea = new Map();
  for (const dup of duplicates) {
    for (const area of requireDuplicateAreas(dup)) {
      if (!byArea.has(area)) byArea.set(area, []);
      byArea.get(area).push(dup);
    }
  }

  lines.push(`**Total**: ${duplicates.length} duplicate code fragments`);
  lines.push("");

  for (const [area, fragments] of byArea.entries()) {
    lines.push(`### ${area} (${fragments.length} fragments)`);
    lines.push("");

    for (const frag of fragments.slice(0, 5)) {
      const locations = requireDuplicateLocations(frag);
      lines.push(`- **Fragment #${frag.id}**: ${frag.tokenCount} tokens, ${frag.lineCount} lines`);
      lines.push(`  - Locations (${locations.length}):`);
      for (const location of locations) {
        lines.push(`    - ${formatDuplicateLocation(frag, location)}`);
      }
      if (frag.hitsChangedScope) {
        lines.push("  - ⚠️ 命中 changed scope");
      }
    }

    if (fragments.length > 5) {
      lines.push(`- *... and ${fragments.length - 5} more fragments*`);
    }
    lines.push("");
  }

  return lines.join("\n");
}

function requireDuplicateAreas(dup: any) {
  if (!Array.isArray(dup.codeAreas) || dup.codeAreas.length === 0) {
    throw new Error(`Duplicate fragment #${dup.id} is missing code areas`);
  }
  return dup.codeAreas;
}

function requireDuplicateLocations(dup: any) {
  if (!Array.isArray(dup.locations) || dup.locations.length === 0) {
    throw new Error(`Duplicate fragment #${dup.id} is missing locations`);
  }
  return dup.locations;
}

function formatDuplicateLocation(dup: any, location: any) {
  if (!location.path || !Number.isInteger(location.startLine) || !Number.isInteger(location.endLine)) {
    throw new Error(`Duplicate fragment #${dup.id} has an incomplete location`);
  }
  if (!location.codeArea || location.codeArea === "unknown") {
    throw new Error(`Duplicate fragment #${dup.id} location is missing code area`);
  }

  const endLine = location.endLine && location.endLine !== location.startLine
    ? `-${location.endLine}`
    : "";

  return `${location.path}:${location.startLine}${endLine} (${location.codeArea})`;
}

export function changedFilesSection(metrics: any) {
  const lines: any[] = [];
  lines.push("## Changed Files Watchlist");
  lines.push("");

  const changed = metrics.fileMetrics.filter((file: any) => file.isChanged);
  if (changed.length === 0) {
    lines.push("*(no changed files in scan scope)*");
    return lines.join("\n");
  }

  lines.push(`**${changed.length} changed files in scan scope:**`);
  lines.push("");

  const rows = [["File", "Area", "Lines", "Complexity"]];
  for (const file of changed.slice(0, 20)) {
    const complexity = file.complexity.value !== null ? String(file.complexity.value) : "n/a";
    rows.push([
      file.path,
      file.codeArea,
      file.lines.toLocaleString(),
      complexity
    ]);
  }
  lines.push(formatTable(rows));

  if (changed.length > 20) {
    lines.push(`*... and ${changed.length - 20} more changed files*`);
  }

  return lines.join("\n");
}

export function trendSection(metrics: any) {
  const lines: any[] = [];
  lines.push("## 趋势比较 (Previous-Code Baseline)");
  lines.push("");

  if (metrics.comparisonStatus === "input-unchanged") {
    lines.push("*(text-only change — code input fingerprint 未变化)*");
    return lines.join("\n");
  }

  if (metrics.comparisonStatus === "baseline-unavailable") {
    lines.push("*(baseline 不可用，无法生成趋势)*");
    return lines.join("\n");
  }

  const trends = metrics.trends || [];
  if (trends.length === 0) {
    lines.push("*(no trend data available)*");
    return lines.join("\n");
  }

  const rows = [["Metric", "Current", "Baseline", "Delta", "Change %"]];
  for (const trend of trends) {
    const deltaStr = trend.delta !== null
      ? (trend.delta >= 0 ? `+${trend.delta}` : `${trend.delta}`)
      : "n/a";
    const pctStr = trend.percentChange !== null
      ? `${trend.percentChange >= 0 ? "+" : ""}${trend.percentChange.toFixed(1)}%`
      : "n/a";

    rows.push([
      trend.metric,
      trend.current !== null ? trend.current.toLocaleString() : "n/a",
      trend.baseline !== null ? trend.baseline.toLocaleString() : "n/a",
      deltaStr,
      pctStr
    ]);
  }
  lines.push(formatTable(rows));

  return lines.join("\n");
}

export function warningsSection(metrics: any) {
  const lines: any[] = [];
  lines.push("## Warnings");
  lines.push("");

  const warnings = metrics.warnings || [];
  if (warnings.length === 0) {
    lines.push("*(no warnings generated)*");
    return lines.join("\n");
  }

  const byLevel = {
    error: warnings.filter((warning: any) => warning.level === "error"),
    warning: warnings.filter((warning: any) => warning.level === "warning"),
    info: warnings.filter((warning: any) => warning.level === "info")
  };

  lines.push(`**Total**: ${warnings.length} warnings (${byLevel.error.length} errors, ${byLevel.warning.length} warnings, ${byLevel.info.length} info)`);
  lines.push("");

  for (const [level, levelWarnings] of Object.entries(byLevel)) {
    if (levelWarnings.length === 0) continue;
    const icon = level === "error" ? "🔴" : level === "warning" ? "🟡" : "ℹ️";
    lines.push(`### ${icon} ${level.toUpperCase()} (${levelWarnings.length})`);
    lines.push("");

    for (const warning of levelWarnings.slice(0, 10)) {
      lines.push(`- **\[${warning.sourceTool}\] ${warning.metric}**: ${warning.message}`);
      if (warning.suggestion) {
        lines.push(`  → ${warning.suggestion}`);
      }
    }

    if (levelWarnings.length > 10) {
      lines.push(`- *... and ${levelWarnings.length - 10} more ${level} warnings*`);
    }
    lines.push("");
  }

  return lines.join("\n");
}
