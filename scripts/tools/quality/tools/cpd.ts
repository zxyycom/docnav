/**
 * PMD CPD 重复代码检测 wrapper。
 *
 * 封装 PMD CPD 调用，按 code area 传递 minimum tokens，
 * 统一输出重复片段、token count、涉及文件、起始行、code area 和排序。
 *
 * PMD 7.25 兼容：使用 --file-list 代替 --files，
 * 版本探测使用 pmd --version（而非 CPD 子命令的 --version）。
 *
 * 来源：openspec/changes/implement-code-quality-observability/tasks.md task 3.4
 */

import { spawnSync } from "node:child_process";
import type { SpawnSyncOptionsWithStringEncoding, SpawnSyncReturns } from "node:child_process";
import { writeFileSync, unlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { randomUUID } from "node:crypto";

import type { DuplicateCodeFragment, DuplicateCodeLocation, ToolConfig } from "../schema.ts";
import { errorMessage } from "../../types.ts";

/**
 * Code area 到 PMD CPD --language 的映射。
 *
 * 注：PMD 7.25 CPD 的 --language 参数值。
 * ecmascript 在某些安装中对 .ts + --file-list 组合不工作，
 * 此时节点脚本 code areas 应回退为跳过并记录原因。
 */
const CODE_AREA_LANGUAGE: Record<string, string | null> = {
  "rust-production": "rust",
  "rust-tests": "rust",
  "node-production-scripts": "ecmascript",
  "node-validation-smoke": "ecmascript",
  "fixtures-examples": null,   // 不传 --language，让 PMD 自动检测
  "generated": null
};

/**
 * 使用 PMD CPD 扫描指定文件，返回重复代码片段指标。
 *
 * PMD CPD 默认使用 XML 输出，我们解析其输出并转换为 DuplicateCodeFragment。
 *
 * PMD 7.25 兼容：
 * - 使用 --file-list 临时文件（写入相对路径，扫描结束后清理）
 * - 传递 --language 以提升检测精确度
 *
 * @param {object} params
 * @param {string[]} params.files - 待扫描文件列表
 * @param {string} params.cwd - 工作目录
 * @param {{ command: string, args: string[] }} params.toolConfig - PMD CPD 工具配置
 * @param {number} params.minimumTokens - 最小 token 阈值
 * @param {string} [params.codeArea] - code area 名称，用于选择 --language
 * @param {boolean} [params.skipIfUnavailable] - 工具不可用时是否静默返回空（默认 false）
 * @returns {{ ok: true, fragments: DuplicateCodeFragment[] } | { ok: false, skipped: boolean, error: string, reason?: string }}
 *
 * @typedef {import('../schema.ts').DuplicateCodeFragment} DuplicateCodeFragment
 */
interface ScanWithCpdOptions {
  codeArea?: string;
  cwd: string;
  files: string[];
  minimumTokens: number;
  skipIfUnavailable?: boolean;
  toolConfig: ToolConfig;
}

type CpdScanResult =
  | { fragments: DuplicateCodeFragment[]; ok: true }
  | { error: string; ok: false; reason?: string; skipped: boolean };

export function scanWithCpd({
  files,
  cwd,
  toolConfig,
  minimumTokens,
  codeArea = "fixtures-examples",
  skipIfUnavailable = false
}: ScanWithCpdOptions): CpdScanResult {
  if (files.length < 2) {
    return { ok: true, fragments: [] };
  }

  // 选择 language
  const language = CODE_AREA_LANGUAGE[codeArea] ?? null;

  // 写入临时 file-list（PMD 7.25 使用 --file-list 而非 --files）
  const fileListPath = join(tmpdir(), `docnav-cpd-filelist-${randomUUID()}.txt`);
  try {
    // 写相对路径（相对于 cwd），每行一个文件
    writeFileSync(fileListPath, files.join("\n"), "utf8");
  } catch (error: unknown) {
    return {
      ok: false,
      skipped: false,
      error: `Failed to write CPD file list: ${errorMessage(error)}`
    };
  }

  const argv = [
    ...toolConfig.args,
    "--minimum-tokens", String(minimumTokens),
    "--format", "xml",
    "--file-list", fileListPath,
    "--skip-lexical-errors"
  ];

  // 添加 --language（如果 code area 支持）
  if (language) {
    argv.push("--language", language);
  }

  const child = spawnPmd(toolConfig.command, argv, {
    cwd,
    encoding: "utf8",
    windowsHide: true,
    maxBuffer: 1024 * 1024 * 64,
    timeout: 600_000
  });

  // 清理临时文件
  tryCleanupFileList(fileListPath);

  if (child.error) {
    // 检查是否是 tool-not-found 错误
    if ((child.error as NodeJS.ErrnoException).code === "ENOENT") {
      if (skipIfUnavailable) {
        return { ok: true, fragments: [] };
      }
      return {
        ok: false,
        skipped: true,
        error: `PMD CPD not found: ${child.error.message}`,
        reason: "tool-unavailable"
      };
    }
    return {
      ok: false,
      skipped: false,
      error: `PMD CPD spawn error: ${child.error.message}`
    };
  }

  if (child.status !== 0 && child.status !== null) {
    const stderr = (child.stderr || "").trim();
    // CPD 即使发现重复代码也可能返回 exit 4（发现重复）
    // exit 0 = no duplicates, exit 4 = duplicates found
    if (child.status === 4) {
      return parseCpdXml(child.stdout || "", cwd);
    } else if (stderr) {
      // PMD 7.25：某些文件类型可能无法分析——视为非阻塞跳过
      if (skipIfUnavailable) {
        return { ok: true, fragments: [] };
      }
      return {
        ok: false,
        skipped: true,
        error: `PMD CPD exit ${child.status}: ${stderr}`,
        reason: "tool-unavailable"
      };
    } else {
      return parseCpdXml(child.stdout || "", cwd);
    }
  }

  const output = child.stdout || "";
  if (!output) {
    return { ok: true, fragments: [] };
  }

  return parseCpdXml(output, cwd);
}

/**
 * 解析 CPD XML 输出。
 *
 * CPD 格式：
 * ```xml
 * <?xml version="1.0" encoding="UTF-8"?>
 * <pmd-cpd>
 *   <duplication lines="10" tokens="50">
 *     <file path="/path/to/file1.rs" line="10" endline="20"/>
 *     <file path="/path/to/file2.rs" line="5" endline="15"/>
 *   </duplication>
 * </pmd-cpd>
 * ```
 *
 * @param {string} xml
 * @param {string} cwd
 * @returns {{ ok: true, fragments: DuplicateCodeFragment[] } | { ok: false, error: string }}
 */
export function parseCpdXml(xml: string, cwd: string): CpdScanResult {
  try {
    const fragments: DuplicateCodeFragment[] = [];
    const dupRegex = /<duplication\b([^>]*)>([\s\S]*?)<\/duplication>/g;
    const fileRegex = /<file\b([^>]*)\/>/g;

    let match;
    let idCounter = 0;

    while ((match = dupRegex.exec(xml)) !== null) {
      const duplicateAttrs = parseXmlAttributes(match[1]);
      const lines = parseIntegerAttribute(duplicateAttrs, "lines");
      const tokens = parseIntegerAttribute(duplicateAttrs, "tokens");
      const inner = match[2];

      /** @type {{ path: string, startLine: number, endLine: number, codeArea: string }[]} */
      const locations: DuplicateCodeLocation[] = [];
      const areaSet = new Set<string>();

      let fileMatch;
      while ((fileMatch = fileRegex.exec(inner)) !== null) {
        const fileAttrs = parseXmlAttributes(fileMatch[1]);
        const rawPath = fileAttrs.get("path");
        const rawLine = fileAttrs.get("line");
        if (!rawPath || !rawLine) {
          throw new Error("CPD XML file entry must include path and line attributes");
        }

        const path = normalizePath(rawPath, cwd);
        const startLine = parseIntegerAttribute(fileAttrs, "line");
        const endLine = fileAttrs.has("endline")
          ? parseIntegerAttribute(fileAttrs, "endline")
          : startLine + lines;

        locations.push({
          path,
          startLine,
          endLine,
          codeArea: "unknown"
        });
      }

      if (locations.length === 0) {
        throw new Error("CPD XML duplication must include at least one file location");
      }

      fragments.push({
        id: ++idCounter,
        tokenCount: tokens,
        lineCount: lines,
        locations,
        codeAreas: Array.from(areaSet),
        hitsChangedScope: false
      });
    }

    // 按 token count 降序
    fragments.sort((a, b) => b.tokenCount - a.tokenCount);

    return { ok: true, fragments };
  } catch (error: unknown) {
    return { ok: false, skipped: false, error: `Failed to parse CPD XML: ${errorMessage(error)}` };
  }
}

/**
 * 查询 PMD 版本。
 *
 * PMD 7.25 兼容：使用 `pmd --version` 进行版本探测；
 * `pmd cpd --version` 在 7.25 中不支持（CPD 需要 --minimum-tokens 才能运行）。
 * CPD 子命令在运行时的可用性由 scanWithCpd 自然验证，错误作为非阻塞跳过处理。
 *
 * @param {object} params
 * @param {string} params.cwd
 * @param {{ command: string, args: string[] }} params.toolConfig
 * @returns {{ ok: true, version: string } | { ok: false, error: string, reason?: string }}
 */
export function getCpdVersion({ cwd, toolConfig }: { cwd: string; toolConfig: ToolConfig }) {
  // PMD 7.25: use `pmd --version` instead of `pmd cpd --version`.
  // CPD subcommand requires --minimum-tokens to run, so version is
  // detected from the PMD launcher. CPD runtime availability is
  // validated naturally by scanWithCpd; errors are non-blocking.
  const child = spawnPmd(toolConfig.command, ["--version"], {
    cwd,
    encoding: "utf8",
    windowsHide: true,
    maxBuffer: 1024 * 1024
  });

  if (child.error) {
    if ((child.error as NodeJS.ErrnoException).code === "ENOENT") {
      return { ok: false, error: "PMD not installed", reason: "tool-unavailable" };
    }
    return { ok: false, error: `PMD version error: ${child.error.message}` };
  }

  const output = (child.stdout || "").trim() || (child.stderr || "").trim();
  if (child.status !== 0) {
    return {
      ok: false,
      error: `PMD --version failed, exit ${child.status}${output ? `: ${output}` : ""}`,
      reason: "tool-unavailable"
    };
  }

  return { ok: true, version: parsePmdVersionOutput(output) };
}

// ── Helpers ───────────────────────────────────────────────────────────

function spawnPmd(
  command: string,
  args: string[],
  options: SpawnSyncOptionsWithStringEncoding
): SpawnSyncReturns<string> {
  return spawnSync(buildPmdShellCommand(command, args), {
    ...options,
    shell: true
  });
}

export function buildPmdShellCommand(command: string, args: string[]): string {
  return [command, ...args].map(quoteShellArg).join(" ");
}

function quoteShellArg(value: string): string {
  const text = String(value);
  if (/^[A-Za-z0-9_./:=@+%\\-]+$/.test(text)) {
    return text;
  }
  return `"${text.replace(/"/g, "\\\"")}"`;
}

export function parsePmdVersionOutput(output: string): string {
  const versionLine = output
    .split(/\r?\n/)
    .map((line) => line.trim())
    .find((line) => /^PMD\s+\d/.test(line));

  if (!versionLine) {
    return "unknown";
  }

  const match = versionLine.match(/^PMD\s+([^\s(]+)/);
  return match ? match[1] : versionLine;
}

function parseXmlAttributes(attributeText: string): Map<string, string> {
  const attrs = new Map<string, string>();
  const attrRegex = /([A-Za-z_:][\w:.-]*)\s*=\s*"([^"]*)"/g;

  let match;
  while ((match = attrRegex.exec(attributeText)) !== null) {
    const [, name, value] = match;
    if (name !== undefined && value !== undefined) {
      attrs.set(name, decodeXmlAttribute(value));
    }
  }

  return attrs;
}

function parseIntegerAttribute(attrs: Map<string, string>, name: string): number {
  const value = attrs.get(name);
  if (value === undefined) {
    throw new Error(`CPD XML attribute "${name}" is required`);
  }
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed)) {
    throw new Error(`CPD XML attribute "${name}" must be an integer`);
  }
  return parsed;
}

function decodeXmlAttribute(value: string): string {
  return value.replace(/&(?:#(\d+)|#x([0-9a-fA-F]+)|amp|quot|apos|lt|gt);/g, (
    entity: string,
    dec: string | undefined,
    hex: string | undefined
  ) => {
    if (dec) return String.fromCodePoint(Number.parseInt(dec, 10));
    if (hex) return String.fromCodePoint(Number.parseInt(hex, 16));
    switch (entity) {
      case "&amp;":
        return "&";
      case "&quot;":
        return "\"";
      case "&apos;":
        return "'";
      case "&lt;":
        return "<";
      case "&gt;":
        return ">";
      default:
        return entity;
    }
  });
}

function normalizePath(filePath: string, cwd: string): string {
  const normalizedPath = filePath.replace(/\\/g, "/");
  const normalizedCwd = cwd.replace(/\\/g, "/").replace(/\/$/, "");
  if (normalizedPath === normalizedCwd) {
    return ".";
  }
  if (normalizedPath.startsWith(`${normalizedCwd}/`)) {
    return normalizedPath.slice(normalizedCwd.length + 1);
  }
  return normalizedPath;
}

/**
 * 尽最大努力清理临时 file-list 文件。
 *
 * @param {string} path
 */
function tryCleanupFileList(path: string): void {
  try {
    unlinkSync(path);
  } catch {
    // best effort — temp dir 会由 OS 清理
  }
}
