import { parse } from "csv-parse/sync";

export function parseCsvRows(csv: string): string[][] {
  const records = parse(csv, {
    bom: true,
    relax_column_count: true,
    skip_empty_lines: true,
    trim: true
  }) as unknown;

  if (!Array.isArray(records) || !records.every(isStringRow)) {
    throw new Error("CSV parser returned non-string rows");
  }

  return records;
}

function isStringRow(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}
