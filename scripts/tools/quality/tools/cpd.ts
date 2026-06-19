/**
 * PMD CPD 重复代码检测 wrapper。
 *
 * 封装 PMD CPD 调用，按 code area 传递 minimum tokens，
 * 统一输出重复片段、token count、涉及文件、起始行、code area 和排序。
 *
 * PMD 7.25 CPD 使用 --file-list 接收扫描输入。
 */

import { spawn, spawnSync } from "node:child_process";
import type { SpawnSyncOptionsWithStringEncoding, SpawnSyncReturns } from "node:child_process";
import { writeFileSync, unlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { randomUUID } from "node:crypto";

import type { DuplicateCodeFragment, DuplicateCodeLocation, ToolConfig } from "../schema.ts";
import { errorMessage } from "../../types.ts";

const CODE_AREA_LANGUAGE: Record<string, string | null> = {
  "rust-production": "rust",
  "rust-tests": "rust",
  "node-production-scripts": "typescript",
  "node-validation-smoke": "typescript",
  "fixtures-examples": null,   // 不传 --language，让 PMD 自动检测
  "generated": null
};

export function getCpdLanguageForCodeArea(codeArea: string): string | null {
  return CODE_AREA_LANGUAGE[codeArea] ?? null;
}

interface ScanWithCpdOptions {
  codeArea?: string;
  cwd: string;
  files: string[];
  minimumTokens: number;
  skipIfUnavailable?: boolean;
  toolConfig: ToolConfig;
}

export type CpdScanResult =
  | { fragments: DuplicateCodeFragment[]; ok: true }
  | { error: string; ok: false; reason?: string; skipped: boolean };

/**
 * 使用 PMD CPD 扫描指定文件，返回重复代码片段指标。
 *
 * CPD 扫描失败时返回显式 error；调用方决定 skipped 是否阻断。
 */
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

  const invocation = prepareCpdInvocation({ files, toolConfig, minimumTokens, codeArea });
  if (!invocation.ok) return invocation.result;

  try {
    const child = spawnPmd(toolConfig.command, invocation.argv, {
      cwd,
      encoding: "utf8",
      windowsHide: true,
      maxBuffer: 1024 * 1024 * 64,
      timeout: 600_000
    });

    return handleCpdProcessResult({
      child,
      cwd,
      skipIfUnavailable
    });
  } finally {
    tryCleanupFileList(invocation.fileListPath);
  }
}

export async function scanWithCpdAsync({
  files,
  cwd,
  toolConfig,
  minimumTokens,
  codeArea = "fixtures-examples",
  skipIfUnavailable = false
}: ScanWithCpdOptions): Promise<CpdScanResult> {
  if (files.length < 2) {
    return { ok: true, fragments: [] };
  }

  const invocation = prepareCpdInvocation({ files, toolConfig, minimumTokens, codeArea });
  if (!invocation.ok) return invocation.result;

  try {
    const child = await spawnPmdAsync(toolConfig.command, invocation.argv, {
      cwd,
      windowsHide: true,
      maxBuffer: 1024 * 1024 * 64,
      timeout: 600_000
    });

    return handleCpdProcessResult({
      child,
      cwd,
      skipIfUnavailable
    });
  } finally {
    tryCleanupFileList(invocation.fileListPath);
  }
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

    fragments.sort((a, b) => b.tokenCount - a.tokenCount);

    return { ok: true, fragments };
  } catch (error: unknown) {
    return { ok: false, skipped: false, error: `Failed to parse CPD XML: ${errorMessage(error)}` };
  }
}

// ── Helpers ───────────────────────────────────────────────────────────

type CpdProcessResult = {
  error?: Error;
  status: number | null;
  stderr: string;
  stdout: string;
};

type CpdInvocation =
  | { argv: string[]; fileListPath: string; ok: true }
  | { ok: false; result: CpdScanResult };

function prepareCpdInvocation({
  files,
  toolConfig,
  minimumTokens,
  codeArea
}: {
  codeArea: string;
  files: string[];
  minimumTokens: number;
  toolConfig: ToolConfig;
}): CpdInvocation {
  const fileListPath = join(tmpdir(), `docnav-cpd-filelist-${randomUUID()}.txt`);
  try {
    writeFileSync(fileListPath, files.join("\n"), "utf8");
  } catch (error: unknown) {
    return {
      ok: false,
      result: {
        ok: false,
        skipped: false,
        error: `Failed to write CPD file list: ${errorMessage(error)}`
      }
    };
  }

  return {
    ok: true,
    fileListPath,
    argv: buildCpdArgs({
      toolConfig,
      minimumTokens,
      fileListPath,
      codeArea
    })
  };
}

function buildCpdArgs({
  toolConfig,
  minimumTokens,
  fileListPath,
  codeArea
}: {
  codeArea: string;
  fileListPath: string;
  minimumTokens: number;
  toolConfig: ToolConfig;
}): string[] {
  const language = CODE_AREA_LANGUAGE[codeArea] ?? null;
  const argv = [
    ...toolConfig.args,
    "--minimum-tokens", String(minimumTokens),
    "--format", "xml",
    "--file-list", fileListPath,
    "--no-fail-on-error"
  ];

  if (language) {
    argv.push("--language", language);
  }

  return argv;
}

function handleCpdProcessResult({
  child,
  cwd,
  skipIfUnavailable
}: {
  child: CpdProcessResult;
  cwd: string;
  skipIfUnavailable: boolean;
}): CpdScanResult {
  if (child.error) {
    if ((child.error as NodeJS.ErrnoException).code === "ENOENT") {
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
    if (child.status === 4) {
      const stdout = child.stdout || "";
      if (!stdout.trim()) {
        return cpdExecutionFailure(child.status, "no output", skipIfUnavailable);
      }
      if (!/<pmd-cpd\b/.test(stdout)) {
        return cpdExecutionFailure(child.status, "missing PMD CPD XML output", skipIfUnavailable);
      }
      return parseCpdXml(stdout, cwd);
    }
    const output = stderr || (child.stdout || "").trim() || "no output";
    return cpdExecutionFailure(child.status, output, skipIfUnavailable);
  }

  const output = child.stdout || "";
  if (!output) {
    return { ok: true, fragments: [] };
  }

  return parseCpdXml(output, cwd);
}

function cpdExecutionFailure(status: number, output: string, skipIfUnavailable: boolean): CpdScanResult {
  return {
    ok: false,
    skipped: true,
    error: `PMD CPD exit ${status}: ${output}`,
    reason: skipIfUnavailable ? "cpd-scan-skipped" : "cpd-execution-error"
  };
}

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

function spawnPmdAsync(
  command: string,
  args: string[],
  options: {
    cwd: string;
    maxBuffer: number;
    timeout: number;
    windowsHide: boolean;
  }
): Promise<CpdProcessResult> {
  return new Promise((resolve) => {
    const child = spawn(buildPmdShellCommand(command, args), {
      cwd: options.cwd,
      shell: true,
      windowsHide: options.windowsHide
    });
    let stdout = "";
    let stderr = "";
    let settled = false;

    const finish = (result: CpdProcessResult) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      resolve(result);
    };

    const append = (stream: "stderr" | "stdout", chunk: Buffer | string) => {
      if (stream === "stdout") {
        stdout += chunk.toString();
      } else {
        stderr += chunk.toString();
      }
      if (stdout.length + stderr.length > options.maxBuffer) {
        child.kill();
        finish({
          status: null,
          stdout,
          stderr,
          error: new Error(`PMD CPD output exceeded maxBuffer ${options.maxBuffer}`)
        });
      }
    };

    const timer = setTimeout(() => {
      child.kill();
      finish({
        status: null,
        stdout,
        stderr,
        error: new Error(`PMD CPD timed out after ${options.timeout}ms`)
      });
    }, options.timeout);

    child.stdout?.on("data", (chunk: Buffer | string) => append("stdout", chunk));
    child.stderr?.on("data", (chunk: Buffer | string) => append("stderr", chunk));
    child.on("error", (error) => finish({ status: null, stdout, stderr, error }));
    child.on("close", (status) => finish({ status, stdout, stderr }));
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

function tryCleanupFileList(path: string): void {
  try {
    unlinkSync(path);
  } catch {
    // 临时文件清理失败不影响扫描结果。
  }
}
