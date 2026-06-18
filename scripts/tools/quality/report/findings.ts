import { formatTable } from "./table.ts";
import type {
  DuplicateCodeFragment,
  DuplicateCodeLocation,
  FileMetric,
  QualityMetrics,
  WarningRecord
} from "../schema.ts";

export function duplicateCodeSection(metrics: QualityMetrics): string {
  const lines: string[] = [];
  lines.push("## 重复代码检测");
  lines.push("");

  const duplicates = metrics.duplicateCode;
  if (duplicates.length === 0) {
    lines.push("✅ 未发现重复代码片段（在配置的 minimum tokens 阈值以上）");
    return lines.join("\n");
  }

  const byArea = new Map<string, DuplicateCodeFragment[]>();
  for (const dup of duplicates) {
    for (const area of requireDuplicateAreas(dup)) {
      if (!byArea.has(area)) byArea.set(area, []);
      byArea.get(area)?.push(dup);
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

function requireDuplicateAreas(dup: DuplicateCodeFragment): string[] {
  if (!Array.isArray(dup.codeAreas) || dup.codeAreas.length === 0) {
    throw new Error(`Duplicate fragment #${dup.id} is missing code areas`);
  }
  return dup.codeAreas;
}

function requireDuplicateLocations(dup: DuplicateCodeFragment): DuplicateCodeLocation[] {
  if (!Array.isArray(dup.locations) || dup.locations.length === 0) {
    throw new Error(`Duplicate fragment #${dup.id} is missing locations`);
  }
  return dup.locations;
}

function formatDuplicateLocation(dup: DuplicateCodeFragment, location: DuplicateCodeLocation): string {
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

export function changedFilesSection(metrics: QualityMetrics, topN = 10): string {
  const lines: string[] = [];
  lines.push("## Changed Files Watchlist");
  lines.push("");

  const changed = metrics.fileMetrics.filter((file) => file.isChanged);
  if (changed.length === 0) {
    lines.push("*(no changed files in scan scope)*");
    return lines.join("\n");
  }

  const ranked = rankChangedFilesByRisk(changed, metrics).slice(0, topN);
  lines.push(`Changed files: ${changed.length} total, ${ranked.length} shown by risk ranking`);

  if (ranked.length === 0) {
    lines.push("");
    lines.push("*(no changed files matched warning, delta, or duplicate-code risk criteria)*");
    return lines.join("\n");
  }

  lines.push("");

  const rows = [["File", "Area", "Lines", "Complexity", "Risk"]];
  for (const { file, reasons } of ranked) {
    const complexity = file.complexity.value !== null ? String(file.complexity.value) : "n/a";
    rows.push([
      file.path,
      file.codeArea,
      file.lines.toLocaleString(),
      complexity,
      reasons.join(", ")
    ]);
  }
  lines.push(formatTable(rows));

  return lines.join("\n");
}

export function warningsSection(metrics: QualityMetrics): string {
  const lines: string[] = [];
  lines.push("## Warnings");
  lines.push("");

  const allWarnings = metrics.warnings?.all || [];
  const changedWarnings = metrics.warnings?.changed || [];
  const regressionWarnings = metrics.warnings?.regressions || [];
  if (allWarnings.length === 0) {
    lines.push("*(no warnings generated)*");
    return lines.join("\n");
  }

  lines.push(
    `**All warnings**: ${allWarnings.length} total ` +
    `(${changedWarnings.length} changed, ${regressionWarnings.length} regressions)`
  );
  lines.push("");
  appendWarningsByLevel(lines, allWarnings, "All Warnings Summary");

  lines.push("### Changed Warnings");
  lines.push("");
  if (changedWarnings.length === 0) {
    lines.push("*(no changed warnings for CI annotation)*");
    return lines.join("\n");
  }

  appendWarningList(lines, changedWarnings.slice(0, 10));
  if (changedWarnings.length > 10) {
    lines.push(`- *... and ${changedWarnings.length - 10} more changed warnings*`);
  }

  return lines.join("\n");
}

function rankChangedFilesByRisk(
  changed: FileMetric[],
  metrics: QualityMetrics
): { file: FileMetric; reasons: string[]; score: number }[] {
  const changedWarningPaths = new Set(
    (metrics.warnings?.all || [])
      .filter((warning) => warning.isChanged)
      .map((warning) => warning.path)
  );
  const deltaWarningPaths = new Set(
    (metrics.warnings?.all || [])
      .filter((warning) => warning.isChanged && warning.deltaValue !== null && warning.deltaValue !== 0)
      .map((warning) => warning.path)
  );
  const duplicatePaths = changedDuplicatePaths(metrics.duplicateCode);

  return changed
    .map((file) => riskRankedFile(file, changedWarningPaths, deltaWarningPaths, duplicatePaths))
    .filter((entry) => entry.score > 0)
    .sort((a, b) => b.score - a.score || b.file.lines - a.file.lines || a.file.path.localeCompare(b.file.path));
}

function riskRankedFile(
  file: FileMetric,
  changedWarningPaths: Set<string>,
  deltaWarningPaths: Set<string>,
  duplicatePaths: Set<string>
): { file: FileMetric; reasons: string[]; score: number } {
  const reasons: string[] = [];
  let score = 0;

  if (changedWarningPaths.has(file.path)) {
    reasons.push("current warning");
    score += 4;
  }
  if (deltaWarningPaths.has(file.path)) {
    reasons.push("delta");
    score += 2;
  }
  if (duplicatePaths.has(file.path)) {
    reasons.push("duplicate code");
    score += 3;
  }

  return { file, reasons, score };
}

function changedDuplicatePaths(duplicates: DuplicateCodeFragment[]): Set<string> {
  const paths = new Set<string>();
  for (const duplicate of duplicates) {
    if (!duplicate.hitsChangedScope) continue;
    for (const location of duplicate.locations) {
      paths.add(location.path);
    }
  }
  return paths;
}

function appendWarningsByLevel(lines: string[], warnings: WarningRecord[], title: string): void {
  const byLevel = {
    error: warnings.filter((warning) => warning.level === "error"),
    warning: warnings.filter((warning) => warning.level === "warning"),
    info: warnings.filter((warning) => warning.level === "info")
  };

  lines.push(`### ${title}`);
  lines.push("");
  for (const [level, levelWarnings] of Object.entries(byLevel)) {
    if (levelWarnings.length === 0) continue;
    const icon = level === "error" ? "🔴" : level === "warning" ? "🟡" : "ℹ️";
    lines.push(`#### ${icon} ${level.toUpperCase()} (${levelWarnings.length})`);
    lines.push("");

    appendWarningList(lines, levelWarnings.slice(0, 10));

    if (levelWarnings.length > 10) {
      lines.push(`- *... and ${levelWarnings.length - 10} more ${level} records*`);
    }
    lines.push("");
  }
}

function appendWarningList(lines: string[], warnings: WarningRecord[]): void {
  for (const warning of warnings) {
    lines.push(`- **[${warning.sourceTool}] ${warning.metric}**: ${warning.message}`);
    if (warning.suggestion) {
      lines.push(`  → ${warning.suggestion}`);
    }
  }
}
