import { formatTable } from "./markdown-table.ts";
import type { QualityMetrics } from "../../model/schema.ts";

export function fileRankings(metrics: QualityMetrics, topN: number): string {
  const lines: string[] = [];
  lines.push(`## Top ${topN} 文件 (按行数)`);
  lines.push("");

  const sorted = metrics.fileMetrics
    .filter((file) => file.codeArea !== "generated")
    .sort((a, b) => b.lines - a.lines || a.path.localeCompare(b.path))
    .slice(0, topN);

  if (sorted.length === 0) {
    lines.push("*(no file data available)*");
    return lines.join("\n");
  }

  const rows = [["#", "File", "Area", "Lines", "Complexity"]];
  sorted.forEach((file, index) => {
    const complexity = file.complexity.value !== null ? String(file.complexity.value) : "n/a";
    rows.push([
      String(index + 1),
      file.path,
      file.codeArea,
      file.lines.toLocaleString(),
      complexity
    ]);
  });
  lines.push(formatTable(rows));

  return lines.join("\n");
}

export function fileComplexityRankings(metrics: QualityMetrics, topN: number): string {
  const lines: string[] = [];
  lines.push(`## Top ${topN} 文件 (按复杂度)`);
  lines.push("");

  const sorted = metrics.fileMetrics
    .filter((file) => file.codeArea !== "generated" && file.complexity.value !== null)
    .slice()
    .sort((a, b) =>
      (b.complexity.value ?? 0) - (a.complexity.value ?? 0) ||
      b.lines - a.lines ||
      a.path.localeCompare(b.path)
    )
    .slice(0, topN);

  if (sorted.length === 0) {
    lines.push("*(no file complexity data available)*");
    return lines.join("\n");
  }

  const rows = [["#", "File", "Area", "Complexity", "Lines", "Source"]];
  sorted.forEach((file, index) => {
    rows.push([
      String(index + 1),
      file.path,
      file.codeArea,
      String(file.complexity.value),
      file.lines.toLocaleString(),
      file.complexity.source
    ]);
  });
  lines.push(formatTable(rows));

  return lines.join("\n");
}

export function functionComplexityRankings(metrics: QualityMetrics, topN: number): string {
  const lines: string[] = [];
  lines.push(`## Top ${topN} 函数 (按圈复杂度)`);
  lines.push("");

  const sorted = metrics.functionMetrics
    .filter((func) => func.cyclomaticComplexity.value !== null)
    .slice()
    .sort((a, b) =>
      (b.cyclomaticComplexity.value ?? 0) - (a.cyclomaticComplexity.value ?? 0) ||
      b.lines - a.lines ||
      functionLocation(a).localeCompare(functionLocation(b))
    )
    .slice(0, topN);

  if (sorted.length === 0) {
    lines.push("*(no function complexity data available)*");
    return lines.join("\n");
  }

  const rows = [["#", "Function", "File", "CC", "Code Lines (NLOC)", "Params"]];
  sorted.forEach((func, index) => {
    rows.push([
      String(index + 1),
      func.name,
      `${func.file}:${func.startLine}`,
      String(func.cyclomaticComplexity.value),
      String(func.lines),
      String(func.parameterCount)
    ]);
  });
  lines.push(formatTable(rows));

  return lines.join("\n");
}

export function functionSizeRankings(metrics: QualityMetrics, topN: number): string {
  const lines: string[] = [];
  lines.push(`## Top ${topN} 函数 (按代码行数 / NLOC)`);
  lines.push("");

  const sorted = metrics.functionMetrics
    .slice()
    .sort((a, b) =>
      b.lines - a.lines ||
      (b.cyclomaticComplexity.value ?? 0) - (a.cyclomaticComplexity.value ?? 0) ||
      functionLocation(a).localeCompare(functionLocation(b))
    )
    .slice(0, topN);

  if (sorted.length === 0) {
    lines.push("*(no function size data available)*");
    return lines.join("\n");
  }

  const rows = [["#", "Function", "File", "Code Lines (NLOC)", "CC", "Params"]];
  sorted.forEach((func, index) => {
    rows.push([
      String(index + 1),
      func.name,
      `${func.file}:${func.startLine}`,
      String(func.lines),
      func.cyclomaticComplexity.value !== null ? String(func.cyclomaticComplexity.value) : "n/a",
      String(func.parameterCount)
    ]);
  });
  lines.push(formatTable(rows));

  return lines.join("\n");
}

function functionLocation(func: QualityMetrics["functionMetrics"][number]): string {
  return `${func.file}:${func.startLine}:${func.name}`;
}
