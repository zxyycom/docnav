import type { FunctionMetric } from "../../../model/schema.ts";
import { parseCsvRows } from "../../../../csv.ts";
import { errorMessage } from "../../../../errors.ts";

export type LizardScanResult =
  | { functions: FunctionMetric[]; ok: true }
  | { error: string; ok: false };

const LIZARD_COLUMNS = {
  nloc: 0,
  ccn: 1,
  parameterCount: 3,
  filePath: 6,
  functionName: 7,
  startLine: 9,
  endLine: 10
} as const;

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

  const nloc = parseOptionalInteger(parts[LIZARD_COLUMNS.nloc]);
  const ccn = parseOptionalInteger(parts[LIZARD_COLUMNS.ccn]);
  const paramCount = parseOptionalInteger(parts[LIZARD_COLUMNS.parameterCount]);
  const startLine = parseOptionalInteger(parts[LIZARD_COLUMNS.startLine]);
  const endLine = parseOptionalInteger(parts[LIZARD_COLUMNS.endLine]);

  if (nloc === null || startLine === null) {
    return null;
  }

  return {
    name: parts[LIZARD_COLUMNS.functionName] || "unknown",
    file: parts[LIZARD_COLUMNS.filePath],
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
  return parts.length >= 11
    && isIntegerText(parts[LIZARD_COLUMNS.startLine])
    && isIntegerText(parts[LIZARD_COLUMNS.endLine]);
}

function isIntegerText(value: string | undefined): boolean {
  return /^-?\d+$/.test(String(value ?? ""));
}
