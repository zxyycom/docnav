import { errorMessage } from "./types.ts";

export type NdjsonRecord<T = unknown> = {
  line: number;
  value: T;
};

export type NdjsonDiagnostic = {
  line: number;
  message: string;
};

export function toNdjson(values: readonly unknown[]): string {
  return values.length === 0 ? "" : `${values.map((value) => JSON.stringify(value)).join("\n")}\n`;
}

export function parseNdjson(content: string): {
  diagnostics: NdjsonDiagnostic[];
  records: NdjsonRecord[];
} {
  const diagnostics: NdjsonDiagnostic[] = [];
  const records: NdjsonRecord[] = [];

  for (const [index, line] of content.split(/\r?\n/).entries()) {
    if (!line.trim()) continue;
    try {
      records.push({ line: index + 1, value: JSON.parse(line) });
    } catch (error: unknown) {
      diagnostics.push({ line: index + 1, message: `invalid JSON: ${errorMessage(error)}` });
    }
  }

  return { diagnostics, records };
}
