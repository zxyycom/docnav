/**
 * scc 仓库/文件级指标 wrapper。
 *
 * 封装 scc 调用，统一输出仓库体量、语言占比、文件行数、文件级复杂度、
 * 路径和排序。
 */

import { spawnSync } from "node:child_process";

import type { FileMetric, LanguageAggregate, ToolConfig } from "../schema.ts";
import { errorMessage } from "../../types.ts";

export const SCC_VERSION = "3.7.0";
export const SCC_VERSION_OUTPUT = `scc version ${SCC_VERSION}`;
export const SCC_BY_FILE_CSV_HEADER = "Language,Provider,Filename,Lines,Code,Comments,Blanks,Complexity,Bytes,ULOC";

interface ScanWithSccOptions {
  cwd: string;
  excludeDirs: string[];
  includePaths: string[];
  toolConfig: ToolConfig;
}

type SccScanResult =
  | { aggregates: { byLanguage: LanguageAggregate[] }; files: FileMetric[]; ok: true }
  | { error: string; ok: false };

export function scanWithScc({ cwd, includePaths, excludeDirs, toolConfig }: ScanWithSccOptions): SccScanResult {
  const argv = buildSccArgs({ includePaths, excludeDirs, toolArgs: toolConfig.args });

  const child = spawnSync(toolConfig.command, argv, {
    cwd,
    encoding: "utf8",
    windowsHide: true,
    maxBuffer: 1024 * 1024 * 64,
    timeout: 300_000
  });

  if (child.error) {
    return {
      ok: false,
      error: `scc spawn error: ${child.error.message}`
    };
  }

  if (child.status !== 0 && child.status !== null) {
    const stderr = (child.stderr || "").trim();
    const stdout = (child.stdout || "").trim();
    return {
      ok: false,
      error: `scc exit ${child.status}: ${stderr || stdout || "no output"}`
    };
  }

  const output = child.stdout || "";
  return parseSccCSV(output, cwd);
}

export function buildSccArgs({
  includePaths,
  excludeDirs,
  toolArgs
}: {
  excludeDirs: string[];
  includePaths: string[];
  toolArgs: string[];
}): string[] {
  const excludeArgs = excludeDirs.flatMap((d) => ["--exclude-dir", d]);
  return [...toolArgs, "--by-file", "--format", "csv", ...excludeArgs, ...includePaths];
}

/**
 * 解析 scc CSV 输出。
 *
 * scc 3.7.0 `--by-file --format csv` 列：
 * Language,Provider,Filename,Lines,Code,Comments,Blanks,Complexity,Bytes,ULOC
 *
 * - Lines 包含所有行（code + comments + blanks）
 * - Complexity 是 scc 的文件级复杂度（非函数级 CC）
 * - ULOC (Usable Lines of Code) 由 3.7.0 输出，但首期不进入稳定 metrics
 */
export function parseSccCSV(csv: string, cwd: string): SccScanResult {
  try {
    const lines = csv.split(/\r?\n/).filter((line) => line.trim().length > 0);
    if (lines.length === 0) {
      return { ok: true, files: [], aggregates: { byLanguage: [] } };
    }

    const headerIdx = lines.findIndex((line) => line.trim() === SCC_BY_FILE_CSV_HEADER);
    if (headerIdx < 0) {
      const observedHeader = lines.find((line) => /^Language,/.test(line)) ?? lines[0];
      return {
        ok: false,
        error: `expected scc ${SCC_VERSION} by-file CSV header "${SCC_BY_FILE_CSV_HEADER}", got "${observedHeader}"`
      };
    }

    const headerLine = lines[headerIdx];
    const headerCols = parseCSVLine(headerLine);

    const colIdx = {
      language: headerCols.indexOf("Language"),
      provider: headerCols.indexOf("Provider"),
      filename: headerCols.indexOf("Filename"),
      lines: headerCols.indexOf("Lines"),
      code: headerCols.indexOf("Code"),
      comments: headerCols.indexOf("Comments"),
      blanks: headerCols.indexOf("Blanks"),
      complexity: headerCols.indexOf("Complexity"),
      bytes: headerCols.indexOf("Bytes"),
      uloc: headerCols.indexOf("ULOC")
    };

    const files: FileMetric[] = [];

    const langMap = new Map<string, LanguageAggregate>();

    for (let i = headerIdx + 1; i < lines.length; i++) {
      const parts = parseCSVLine(lines[i]);
      if (parts.length < Math.max(6, colIdx.filename + 1)) continue;

      const language = parts[colIdx.language] || "";
      const filename = parts[colIdx.filename] || "";
      const path = parts[colIdx.provider] || filename;
      const lineCount = parseInt(parts[colIdx.lines], 10);
      const codeLines = parseInt(parts[colIdx.code], 10);
      const commentLines = parseInt(parts[colIdx.comments], 10);
      const blankLines = parseInt(parts[colIdx.blanks], 10);
      const complexity = colIdx.complexity >= 0 ? parseInt(parts[colIdx.complexity], 10) : NaN;

      if (isNaN(lineCount) || !filename) continue;

      files.push({
        path: normalizePath(path, cwd),
        language,
        codeArea: "unknown",
        lines: lineCount,
        codeLines: isNaN(codeLines) ? 0 : codeLines,
        commentLines: isNaN(commentLines) ? 0 : commentLines,
        blankLines: isNaN(blankLines) ? 0 : blankLines,
        complexity: {
          value: isNaN(complexity) ? null : complexity,
          source: "scc"
        },
        isChanged: false
      });

      const existing = langMap.get(language);
      if (existing) {
        existing.files++;
        existing.lines += lineCount;
        existing.codeLines += isNaN(codeLines) ? 0 : codeLines;
        existing.commentLines += isNaN(commentLines) ? 0 : commentLines;
        existing.blankLines += isNaN(blankLines) ? 0 : blankLines;
      } else {
        langMap.set(language, {
          language,
          files: 1,
          lines: lineCount,
          codeLines: isNaN(codeLines) ? 0 : codeLines,
          commentLines: isNaN(commentLines) ? 0 : commentLines,
          blankLines: isNaN(blankLines) ? 0 : blankLines,
          complexitySource: "scc"
        });
      }
    }

    files.sort((a, b) => b.lines - a.lines);

    const byLanguage = Array.from(langMap.values()).sort(
      (a, b) => b.lines - a.lines
    );

    return { ok: true, files, aggregates: { byLanguage } };
  } catch (error: unknown) {
    return { ok: false, error: `Failed to parse scc CSV: ${errorMessage(error)}` };
  }
}

// ── Helpers ───────────────────────────────────────────────────────────

function parseCSVLine(line: string): string[] {
  const result: string[] = [];
  let current = "";
  let inQuotes = false;

  for (let i = 0; i < line.length; i++) {
    const ch = line[i];
    if (inQuotes) {
      if (ch === '"') {
        if (i + 1 < line.length && line[i + 1] === '"') {
          current += '"';
          i++;
        } else {
          inQuotes = false;
        }
      } else {
        current += ch;
      }
    } else {
      if (ch === '"') {
        inQuotes = true;
      } else if (ch === ",") {
        result.push(current.trim());
        current = "";
      } else {
        current += ch;
      }
    }
  }
  result.push(current.trim());
  return result;
}

function normalizePath(filePath: string, _cwd: string): string {
  return filePath.replace(/\\/g, "/");
}
