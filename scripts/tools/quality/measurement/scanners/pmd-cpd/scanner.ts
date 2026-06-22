/**
 * PMD CPD 重复代码检测 wrapper。
 *
 * 封装 PMD CPD 调用，按 code area 传递 minimum tokens，
 * 统一输出重复片段、token count、涉及文件、起始行、code area 和排序。
 *
 * PMD 7.25 CPD 使用 --file-list 接收扫描输入。
 */

import { writeFileSync, unlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { randomUUID } from "node:crypto";

import type { ToolConfig } from "../../../model/schema.ts";
import { runProcess, runProcessSync } from "../../../../process.ts";
import { errorMessage } from "../../../../errors.ts";
import { handlePmdCpdProcessResult } from "./process-result.ts";
import type { PmdCpdScanResult } from "./types.ts";

export type { PmdCpdScanResult } from "./types.ts";
export { parsePmdCpdXml } from "./xml-report.ts";

const CODE_AREA_LANGUAGE: Record<string, string | null> = {
  "rust-production": "rust",
  "rust-tests": "rust",
  "typescript-production-scripts": "typescript",
  "typescript-validation-smoke": "typescript",
  "fixtures-examples": null,   // 不传 --language，让 PMD 自动检测
  "generated": null
};

const CPD_PROCESS_MAX_BUFFER = 1024 * 1024 * 64;
const CPD_PROCESS_TIMEOUT_MS = 600_000;

export function getPmdCpdLanguageForCodeArea(codeArea: string): string | null {
  return CODE_AREA_LANGUAGE[codeArea] ?? null;
}

interface ScanWithPmdCpdOptions {
  codeArea?: string;
  cwd: string;
  files: string[];
  minimumTokens: number;
  skipIfUnavailable?: boolean;
  toolConfig: ToolConfig;
}

type PreparedPmdCpdInvocation = { argv: string[]; fileListPath: string; ok: true };

type PmdCpdInvocation =
  | PreparedPmdCpdInvocation
  | { ok: false; result: PmdCpdScanResult };

type ExecutablePmdCpdScan = {
  cwd: string;
  invocation: PreparedPmdCpdInvocation;
  ok: true;
  skipIfUnavailable: boolean;
  toolConfig: ToolConfig;
};

type PreparedPmdCpdScan =
  | ExecutablePmdCpdScan
  | { ok: false; result: PmdCpdScanResult };

/**
 * 使用 PMD CPD 扫描指定文件，返回重复代码片段指标。
 *
 * CPD 扫描失败时返回显式 error；调用方决定 skipped 是否阻断。
 */
export function scanWithPmdCpd(options: ScanWithPmdCpdOptions): PmdCpdScanResult {
  const scan = preparePmdCpdScan(options);
  if (!scan.ok) return scan.result;

  try {
    const child = runProcessSync(scan.toolConfig.command, scan.invocation.argv, {
      cwd: scan.cwd,
      encoding: "utf8",
      windowsHide: true,
      maxBuffer: CPD_PROCESS_MAX_BUFFER,
      timeout: CPD_PROCESS_TIMEOUT_MS
    });

    return handlePmdCpdProcessResult({
      child,
      cwd: scan.cwd,
      skipIfUnavailable: scan.skipIfUnavailable
    });
  } finally {
    tryCleanupFileList(scan.invocation.fileListPath);
  }
}

export async function scanWithPmdCpdAsync(options: ScanWithPmdCpdOptions): Promise<PmdCpdScanResult> {
  const scan = preparePmdCpdScan(options);
  if (!scan.ok) return scan.result;

  try {
    const child = await runProcess({
      args: scan.invocation.argv,
      command: scan.toolConfig.command,
      cwd: scan.cwd,
      label: "PMD CPD",
      maxBuffer: CPD_PROCESS_MAX_BUFFER,
      timeout: CPD_PROCESS_TIMEOUT_MS,
      windowsHide: true
    });

    return handlePmdCpdProcessResult({
      child,
      cwd: scan.cwd,
      skipIfUnavailable: scan.skipIfUnavailable
    });
  } finally {
    tryCleanupFileList(scan.invocation.fileListPath);
  }
}

function preparePmdCpdScan(options: ScanWithPmdCpdOptions): PreparedPmdCpdScan {
  const {
    files,
    cwd,
    toolConfig,
    minimumTokens,
    codeArea = "fixtures-examples",
    skipIfUnavailable = false
  } = options;

  if (files.length < 2) {
    return { ok: false, result: { ok: true, fragments: [] } };
  }

  const invocation = preparePmdCpdInvocation({ files, toolConfig, minimumTokens, codeArea });
  if (!invocation.ok) return { ok: false, result: invocation.result };

  return {
    ok: true,
    cwd,
    toolConfig,
    skipIfUnavailable,
    invocation
  };
}

// ── Helpers ───────────────────────────────────────────────────────────

function preparePmdCpdInvocation({
  files,
  toolConfig,
  minimumTokens,
  codeArea
}: {
  codeArea: string;
  files: string[];
  minimumTokens: number;
  toolConfig: ToolConfig;
}): PmdCpdInvocation {
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
    argv: buildPmdCpdArgs({
      toolConfig,
      minimumTokens,
      fileListPath,
      codeArea
    })
  };
}

function buildPmdCpdArgs({
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

function tryCleanupFileList(path: string): void {
  try {
    unlinkSync(path);
  } catch {
    // 临时文件清理失败不影响扫描结果。
  }
}
