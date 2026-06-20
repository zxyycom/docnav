import { formatTable } from "./markdown-table.ts";
import type { QualityMetrics } from "../../model/schema.ts";

export function fileRankings(metrics: QualityMetrics, topN: number): string {
  return renderRanking({
    title: `## Top ${topN} 文件 (按行数)`,
    emptyMessage: "*(no file data available)*",
    headers: ["#", "File", "Area", "Lines", "Complexity"],
    items: topFilesByLines(metrics, topN),
    row: (file, index) => [
      String(index + 1),
      file.path,
      file.codeArea,
      file.lines.toLocaleString(),
      file.complexity.value !== null ? String(file.complexity.value) : "n/a"
    ]
  });
}

export function fileComplexityRankings(metrics: QualityMetrics, topN: number): string {
  return renderRanking({
    title: `## Top ${topN} 文件 (按复杂度)`,
    emptyMessage: "*(no file complexity data available)*",
    headers: ["#", "File", "Area", "Complexity", "Lines", "Source"],
    items: topFilesByComplexity(metrics, topN),
    row: (file, index) => [
      String(index + 1),
      file.path,
      file.codeArea,
      String(file.complexity.value),
      file.lines.toLocaleString(),
      file.complexity.source
    ]
  });
}

export function functionComplexityRankings(metrics: QualityMetrics, topN: number): string {
  return renderRanking({
    title: `## Top ${topN} 函数 (按圈复杂度)`,
    emptyMessage: "*(no function complexity data available)*",
    headers: ["#", "Function", "File", "CC", "Code Lines (NLOC)", "Params"],
    items: topFunctionsByComplexity(metrics, topN),
    row: (func, index) => [
      String(index + 1),
      func.name,
      `${func.file}:${func.startLine}`,
      String(func.cyclomaticComplexity.value),
      String(func.lines),
      String(func.parameterCount)
    ]
  });
}

export function functionSizeRankings(metrics: QualityMetrics, topN: number): string {
  return renderRanking({
    title: `## Top ${topN} 函数 (按代码行数 / NLOC)`,
    emptyMessage: "*(no function size data available)*",
    headers: ["#", "Function", "File", "Code Lines (NLOC)", "CC", "Params"],
    items: topFunctionsBySize(metrics, topN),
    row: (func, index) => [
      String(index + 1),
      func.name,
      `${func.file}:${func.startLine}`,
      String(func.lines),
      func.cyclomaticComplexity.value !== null ? String(func.cyclomaticComplexity.value) : "n/a",
      String(func.parameterCount)
    ]
  });
}

type FileMetric = QualityMetrics["fileMetrics"][number];
type FunctionMetric = QualityMetrics["functionMetrics"][number];

function renderRanking<T>({
  title,
  emptyMessage,
  headers,
  items,
  row
}: {
  emptyMessage: string;
  headers: string[];
  items: readonly T[];
  row: (item: T, index: number) => string[];
  title: string;
}): string {
  const lines = [title, ""];
  if (items.length === 0) {
    lines.push(emptyMessage);
    return lines.join("\n");
  }

  lines.push(formatTable([headers, ...items.map(row)]));
  return lines.join("\n");
}

function topFilesByLines(metrics: QualityMetrics, topN: number): FileMetric[] {
  return metrics.fileMetrics
    .filter((file) => file.codeArea !== "generated")
    .sort((a, b) => b.lines - a.lines || a.path.localeCompare(b.path))
    .slice(0, topN);
}

function topFilesByComplexity(metrics: QualityMetrics, topN: number): FileMetric[] {
  return metrics.fileMetrics
    .filter((file) => file.codeArea !== "generated" && file.complexity.value !== null)
    .slice()
    .sort((a, b) =>
      (b.complexity.value ?? 0) - (a.complexity.value ?? 0) ||
      b.lines - a.lines ||
      a.path.localeCompare(b.path)
    )
    .slice(0, topN);
}

function topFunctionsByComplexity(metrics: QualityMetrics, topN: number): FunctionMetric[] {
  return metrics.functionMetrics
    .filter((func) => func.cyclomaticComplexity.value !== null)
    .slice()
    .sort((a, b) =>
      (b.cyclomaticComplexity.value ?? 0) - (a.cyclomaticComplexity.value ?? 0) ||
      b.lines - a.lines ||
      functionLocation(a).localeCompare(functionLocation(b))
    )
    .slice(0, topN);
}

function topFunctionsBySize(metrics: QualityMetrics, topN: number): FunctionMetric[] {
  return metrics.functionMetrics
    .slice()
    .sort((a, b) =>
      b.lines - a.lines ||
      (b.cyclomaticComplexity.value ?? 0) - (a.cyclomaticComplexity.value ?? 0) ||
      functionLocation(a).localeCompare(functionLocation(b))
    )
    .slice(0, topN);
}

function functionLocation(func: QualityMetrics["functionMetrics"][number]): string {
  return `${func.file}:${func.startLine}:${func.name}`;
}
