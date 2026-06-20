import type { FileMetric, LanguageAggregate } from "../../../model/schema.ts";
import { parseCsvRows } from "../../../../csv.ts";
import { errorMessage } from "../../../../errors.ts";
import { toSlashPath } from "../../../../path.ts";

export const SCC_VERSION = "3.7.0";
export const SCC_VERSION_OUTPUT = `scc version ${SCC_VERSION}`;
export const SCC_BY_FILE_CSV_HEADER = "Language,Provider,Filename,Lines,Code,Comments,Blanks,Complexity,Bytes,ULOC";

export type SccScanResult =
  | { aggregates: { byLanguage: LanguageAggregate[] }; files: FileMetric[]; ok: true }
  | { error: string; ok: false };

interface SccColumnIndexes {
  blanks: number;
  code: number;
  comments: number;
  complexity: number;
  filename: number;
  language: number;
  lines: number;
  provider: number;
}

type ParsedSccFileMetric = FileMetric & Required<Pick<FileMetric, "blankLines" | "codeLines" | "commentLines">>;

/**
 * 解析 scc CSV 输出。
 *
 * scc 3.7.0 `--by-file --format csv` 列：
 * Language,Provider,Filename,Lines,Code,Comments,Blanks,Complexity,Bytes,ULOC
 *
 * - Lines 包含所有行（code + comments + blanks）
 * - Code 是文件级代码行数，用于文件大小 warning
 * - Complexity 是 scc 的文件级复杂度（非函数级 CC）
 * - ULOC (Usable Lines of Code) 由 3.7.0 输出，但首期不进入稳定 metrics
 */
export function parseSccCSV(csv: string, _cwd: string): SccScanResult {
  try {
    const rows = parseCsvRows(csv);
    if (rows.length === 0) {
      return { ok: true, files: [], aggregates: { byLanguage: [] } };
    }

    const headerIdx = findSccHeaderIndex(rows);
    if (headerIdx < 0) {
      return {
        ok: false,
        error: `expected scc ${SCC_VERSION} by-file CSV header "${SCC_BY_FILE_CSV_HEADER}", got "${observedSccHeader(rows)}"`
      };
    }

    const columns = sccColumnIndexes(rows[headerIdx] ?? []);
    const parsed = parseSccMetrics(rows.slice(headerIdx + 1), columns);
    return { ok: true, files: parsed.files, aggregates: { byLanguage: parsed.byLanguage } };
  } catch (error: unknown) {
    return { ok: false, error: `Failed to parse scc CSV: ${errorMessage(error)}` };
  }
}

function findSccHeaderIndex(rows: string[][]): number {
  const expectedHeader = SCC_BY_FILE_CSV_HEADER.split(",");
  return rows.findIndex((row) => isCsvRow(row, expectedHeader));
}

function observedSccHeader(rows: string[][]): string {
  return rows.find((row) => row[0] === "Language")?.join(",") ?? rows[0]?.join(",") ?? "";
}

function sccColumnIndexes(headerCols: string[]): SccColumnIndexes {
  return {
    language: headerCols.indexOf("Language"),
    provider: headerCols.indexOf("Provider"),
    filename: headerCols.indexOf("Filename"),
    lines: headerCols.indexOf("Lines"),
    code: headerCols.indexOf("Code"),
    comments: headerCols.indexOf("Comments"),
    blanks: headerCols.indexOf("Blanks"),
    complexity: headerCols.indexOf("Complexity")
  };
}

function parseSccMetrics(rows: string[][], columns: SccColumnIndexes): {
  byLanguage: LanguageAggregate[];
  files: FileMetric[];
} {
  const files: FileMetric[] = [];
  const langMap = new Map<string, LanguageAggregate>();

  for (const row of rows) {
    const metric = parseSccFileMetric(row, columns);
    if (!metric) continue;

    files.push(metric);
    addLanguageMetric(langMap, metric);
  }

  files.sort((a, b) => b.lines - a.lines);
  const byLanguage = Array.from(langMap.values()).sort((a, b) => b.lines - a.lines);
  return { files, byLanguage };
}

function parseSccFileMetric(parts: string[], columns: SccColumnIndexes): ParsedSccFileMetric | null {
  if (parts.length < Math.max(6, columns.filename + 1)) return null;

  const language = parts[columns.language] || "";
  const filename = parts[columns.filename] || "";
  const lineCount = parseInt(parts[columns.lines], 10);
  if (isNaN(lineCount) || !filename) return null;

  const path = parts[columns.provider] || filename;
  const codeLines = parseOptionalInt(parts[columns.code]);
  const commentLines = parseOptionalInt(parts[columns.comments]);
  const blankLines = parseOptionalInt(parts[columns.blanks]);
  const complexity = columns.complexity >= 0
    ? parseInt(parts[columns.complexity], 10)
    : NaN;

  return {
    path: toSlashPath(path),
    language,
    codeArea: "unknown",
    lines: lineCount,
    codeLines,
    commentLines,
    blankLines,
    complexity: {
      value: isNaN(complexity) ? null : complexity,
      source: "scc"
    },
    isChanged: false
  };
}

function addLanguageMetric(langMap: Map<string, LanguageAggregate>, metric: ParsedSccFileMetric): void {
  const existing = langMap.get(metric.language);
  if (existing) {
    incrementLanguageAggregate(existing, metric);
    return;
  }

  langMap.set(metric.language, createLanguageAggregate(metric));
}

function incrementLanguageAggregate(
  existing: LanguageAggregate,
  metric: ParsedSccFileMetric
): void {
  existing.files++;
  existing.lines += metric.lines;
  existing.codeLines += metric.codeLines;
  existing.commentLines += metric.commentLines;
  existing.blankLines += metric.blankLines;
}

function createLanguageAggregate(metric: ParsedSccFileMetric): LanguageAggregate {
  return {
    language: metric.language,
    files: 1,
    lines: metric.lines,
    codeLines: metric.codeLines,
    commentLines: metric.commentLines,
    blankLines: metric.blankLines,
    complexitySource: "scc"
  };
}

function parseOptionalInt(value: string): number {
  const parsed = parseInt(value, 10);
  return isNaN(parsed) ? 0 : parsed;
}

function isCsvRow(row: string[], expected: string[]): boolean {
  return row.length === expected.length && row.every((value, index) => value === expected[index]);
}
