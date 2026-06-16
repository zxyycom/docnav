/**
 * scc 仓库/文件级指标 wrapper。
 *
 * 封装 scc 调用，统一输出仓库体量、语言占比、文件行数、文件级复杂度、
 * 路径和排序。
 *
 * 来源：openspec/changes/implement-code-quality-observability/tasks.md task 3.3
 */

import { spawnSync } from "node:child_process";

export const SCC_VERSION = "3.7.0";
export const SCC_VERSION_OUTPUT = `scc version ${SCC_VERSION}`;
export const SCC_BY_FILE_CSV_HEADER = "Language,Provider,Filename,Lines,Code,Comments,Blanks,Complexity,Bytes,ULOC";

/**
 * 使用 scc 扫描仓库，返回文件级指标。
 *
 * @param {object} params
 * @param {string} params.cwd - 工作目录
 * @param {string[]} params.includePaths - 纳入路径模式
 * @param {string[]} params.excludeDirs - 排除目录
 * @param {{ command: string, args: string[] }} params.toolConfig - scc 工具配置
 * @returns {{ ok: true, files: FileMetric[], aggregates: { byLanguage: LanguageAggregate[] } }
 *          | { ok: false, error: string }}
 *
 * @typedef {import('../schema.mjs').FileMetric} FileMetric
 * @typedef {import('../schema.mjs').LanguageAggregate} LanguageAggregate
 */
export function scanWithScc({ cwd, includePaths, excludeDirs, toolConfig }) {
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

export function buildSccArgs({ includePaths, excludeDirs, toolArgs }) {
  const excludeArgs = excludeDirs.flatMap((d) => ["--exclude-dir", d]);
  return [...toolArgs, "--by-file", "--format", "csv", ...excludeArgs, ...includePaths];
}

/**
 * 解析 scc CSV 输出。
 *
 * scc 3.7.0 `--by-file --format csv` 列：
 * Language,Provider,Filename,Lines,Code,Comments,Blanks,Complexity,Bytes,ULOC
 *
 * 注：
 * - Lines 包含所有行（code + comments + blanks）
 * - Complexity 是 scc 的文件级复杂度（非函数级 CC）
 * - ULOC (Usable Lines of Code) 由 3.7.0 输出，但首期不进入稳定 metrics
 *
 * @param {string} csv
 * @param {string} cwd
 * @returns {{ ok: true, files: FileMetric[], aggregates: { byLanguage: LanguageAggregate[] } }
 *          | { ok: false, error: string }}
 */
export function parseSccCSV(csv, cwd) {
  try {
    const lines = csv.split(/\r?\n/).filter((l) => l.trim().length > 0);
    if (lines.length === 0) {
      return { ok: true, files: [], aggregates: { byLanguage: [] } };
    }

    const headerIdx = lines.findIndex((l) => l.trim() === SCC_BY_FILE_CSV_HEADER);
    if (headerIdx < 0) {
      const observedHeader = lines.find((l) => /^Language,/.test(l)) ?? lines[0];
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

    /** @type {FileMetric[]} */
    const files = [];

    /** @type {Map<string, LanguageAggregate>} */
    const langMap = new Map();

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
      // parts[colIdx.bytes] = bytes，不强制使用
      // parts[colIdx.uloc] = usable lines of code，不强制使用

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

      // 聚合
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

    // 按行数降序
    files.sort((a, b) => b.lines - a.lines);

    const byLanguage = Array.from(langMap.values()).sort(
      (a, b) => b.lines - a.lines
    );

    return { ok: true, files, aggregates: { byLanguage } };
  } catch (err) {
    return { ok: false, error: `Failed to parse scc CSV: ${err.message}` };
  }
}

/**
 * 查询 scc 版本。
 *
 * @param {object} params
 * @param {string} params.cwd
 * @param {{ command: string, args: string[] }} params.toolConfig
 * @returns {{ ok: true, version: string } | { ok: false, error: string, reason?: string }}
 */
export function getSccVersion({ cwd, toolConfig }) {
  const child = spawnSync(toolConfig.command, [...toolConfig.args, "--version"], {
    cwd,
    encoding: "utf8",
    windowsHide: true,
    maxBuffer: 1024 * 1024
  });

  if (child.error) {
    if (child.error.code === "ENOENT") {
      return { ok: false, error: `scc not installed: ${child.error.message}`, reason: "tool-unavailable" };
    }
    return { ok: false, error: `scc version error: ${child.error.message}`, reason: "execution-error" };
  }

  const ver = (child.stdout || "").trim() || (child.stderr || "").trim();
  if (child.status !== 0) {
    const failure = typeof child.status === "number"
      ? `exit ${child.status}`
      : `signal ${child.signal || "unknown"}`;
    return {
      ok: false,
      error: `scc --version failed, ${failure}${ver ? `: ${ver}` : ""}`,
      reason: "execution-error"
    };
  }

  if (ver !== SCC_VERSION_OUTPUT) {
    return {
      ok: false,
      error: `expected ${SCC_VERSION_OUTPUT}, got "${ver || "unknown"}"`,
      reason: "contract-error"
    };
  }

  return { ok: true, version: ver };
}

// ── Helpers ───────────────────────────────────────────────────────────

function parseCSVLine(line) {
  const result = [];
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

function normalizePath(filePath, cwd) {
  // 将路径标准化为相对仓库根的路径
  const rel = filePath.replace(/\\/g, "/");
  // scc 输出中路径通常已经是相对路径
  return rel;
}
