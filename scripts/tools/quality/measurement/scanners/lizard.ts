/**
 * Lizard 函数级指标 wrapper。
 *
 * 封装 Lizard 调用，统一输出函数名称、所属文件、函数行数、参数数量、
 * 圈复杂度、路径和排序。
 */

import type { FunctionMetric, ToolConfig } from "../../model/schema.ts";
import { parseCsvRows } from "../../../csv.ts";
import { runProcessSync } from "../../../process.ts";
import { errorMessage } from "../../../errors.ts";

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

  const child = runProcessSync(toolConfig.command, argv, {
    cwd,
    timeout: 300_000
  });

  if (child.error) {
    return {
      ok: false,
      error: `lizard process error: ${child.error.message}`
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
    const functions: FunctionMetric[] = [];

    for (const row of lizardDataRows(parseCsvRows(csv))) {
      const metric = functionMetricFromLizardRow(row);
      if (metric) {
        functions.push(metric);
      }
    }

    functions.sort(compareFunctionMetrics);

    return { ok: true, functions };
  } catch (error: unknown) {
    return { ok: false, error: `Failed to parse lizard CSV: ${errorMessage(error)}` };
  }
}

function lizardDataRows(rows: string[][]): string[][] {
  const header = rows[0] ?? [];
  return header.includes("NLOC") && header.includes("CCN") ? rows.slice(1) : rows;
}

function functionMetricFromLizardRow(parts: string[]): FunctionMetric | null {
  if (!isLizard123Row(parts)) {
    return null;
  }

  const nloc = parseOptionalInteger(parts[0]);
  const ccn = parseOptionalInteger(parts[1]);
  const paramCount = parseOptionalInteger(parts[3]);
  const startLine = parseOptionalInteger(parts[9]);
  const endLine = parseOptionalInteger(parts[10]);

  if (nloc === null || startLine === null) {
    return null;
  }

  return {
    name: parts[7] || "unknown",
    file: parts[6],
    codeArea: "unknown",
    startLine,
    endLine: endLine ?? startLine,
    lines: nloc,
    parameterCount: paramCount ?? 0,
    cyclomaticComplexity: {
      value: ccn,
      source: "lizard"
    },
    isChanged: false
  };
}

function compareFunctionMetrics(a: FunctionMetric, b: FunctionMetric): number {
  const ccDiff = (b.cyclomaticComplexity.value ?? 0) - (a.cyclomaticComplexity.value ?? 0);
  if (ccDiff !== 0) return ccDiff;
  return b.lines - a.lines;
}

function parseOptionalInteger(value: string | undefined): number | null {
  const parsed = parseInt(String(value ?? ""), 10);
  return isNaN(parsed) ? null : parsed;
}

function isLizard123Row(parts: string[]): boolean {
  return parts.length >= 11 && isIntegerText(parts[9]) && isIntegerText(parts[10]);
}

function isIntegerText(value: string | undefined): boolean {
  return /^-?\d+$/.test(String(value ?? ""));
}
