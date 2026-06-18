/**
 * Lizard 函数级指标 wrapper。
 *
 * 封装 Lizard 调用，统一输出函数名称、所属文件、函数行数、参数数量、
 * 圈复杂度、路径和排序。
 */

import { spawnSync } from "node:child_process";

import type { FunctionMetric, ToolConfig } from "../schema.ts";
import { errorMessage } from "../../types.ts";

interface ScanWithLizardOptions {
  cwd: string;
  files: string[];
  toolConfig: ToolConfig;
}

type LizardScanResult =
  | { functions: FunctionMetric[]; ok: true }
  | { error: string; ok: false };

export function scanWithLizard({ files, cwd, toolConfig }: ScanWithLizardOptions): LizardScanResult {
  if (files.length === 0) {
    return { ok: true, functions: [] };
  }

  const argv = [...toolConfig.args, ...files, "--csv"];

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
      error: `lizard spawn error: ${child.error.message}`
    };
  }

  if (child.status !== 0 && child.status !== null) {
    const stderr = (child.stderr || "").trim();
    return {
      ok: false,
      error: `lizard exit ${child.status}: ${stderr || "command succeeded but returned non-zero"}`
    };
  }

  const output = child.stdout || "";
  return parseLizardCSV(output);
}

/**
 * 将 Lizard CSV 输出解析为 FunctionMetric 数组。
 *
 * Lizard 1.23 CSV 列（--csv）：
 * NLOC,CCN,token count,parameter count,length,location,file path,function name,long name,start line,end line
 */
export function parseLizardCSV(csv: string): LizardScanResult {
  try {
    const lines = csv.split(/\r?\n/).filter((line) => line.trim().length > 0);
    if (lines.length === 0) {
      return { ok: true, functions: [] };
    }

    const header = lines[0];
    if (header.includes("NLOC") && header.includes("CCN")) {
      lines.shift(); // remove header line
    }

    const functions: FunctionMetric[] = [];

    for (const line of lines) {
      const parts = parseCSVLine(line);
      if (!isLizard123Row(parts)) continue;

      const nloc = parseInt(parts[0], 10);
      const ccn = parseInt(parts[1], 10);
      const paramCount = parseInt(parts[3], 10);
      const startLine = parseInt(parts[9], 10);
      const endLine = parseInt(parts[10], 10);
      const filePath = parts[6];
      const funcName = parts[7];

      if (isNaN(nloc) || isNaN(startLine)) continue;

      functions.push({
        name: funcName || "unknown",
        file: filePath,
        codeArea: "unknown",
        startLine,
        endLine: isNaN(endLine) ? startLine : endLine,
        lines: nloc,
        parameterCount: isNaN(paramCount) ? 0 : paramCount,
        cyclomaticComplexity: {
          value: isNaN(ccn) ? null : ccn,
          source: "lizard"
        },
        isChanged: false
      });
    }

    functions.sort((a, b) => {
      const ccDiff = (b.cyclomaticComplexity.value ?? 0) - (a.cyclomaticComplexity.value ?? 0);
      if (ccDiff !== 0) return ccDiff;
      return b.lines - a.lines;
    });

    return { ok: true, functions };
  } catch (error: unknown) {
    return { ok: false, error: `Failed to parse lizard CSV: ${errorMessage(error)}` };
  }
}

function isLizard123Row(parts: string[]): boolean {
  return parts.length >= 11 && isIntegerText(parts[9]) && isIntegerText(parts[10]);
}

function isIntegerText(value: string | undefined): boolean {
  return /^-?\d+$/.test(String(value ?? ""));
}

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
