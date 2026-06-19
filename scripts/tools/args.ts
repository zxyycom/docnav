import { parseArgs } from "node:util";
import type { ParseArgsOptionsConfig } from "node:util";

export type ScriptArgValues = Record<string, boolean | string | string[] | undefined>;

export type ParsedScriptArgs = {
  positionals: string[];
  tokens: ScriptArgToken[];
  values: ScriptArgValues;
};

export type ScriptArgToken = {
  index: number;
  inlineValue?: boolean;
  kind: "option" | "option-terminator" | "positional";
  name?: string;
  rawName?: string;
  value?: string;
};

export function parseScriptArgs({
  allowPositionals = false,
  args,
  options
}: {
  allowPositionals?: boolean;
  args: readonly string[];
  options: ParseArgsOptionsConfig;
}): ParsedScriptArgs {
  const result = parseArgs({
    allowPositionals,
    args,
    options,
    strict: true,
    tokens: true
  }) as unknown as ParsedScriptArgs;

  return {
    positionals: result.positionals,
    tokens: result.tokens,
    values: result.values as ScriptArgValues
  };
}

export function stringOption(values: ParsedScriptArgs["values"], name: string): string | undefined {
  const value = values[name];
  return typeof value === "string" ? value : undefined;
}

export function stringArrayOption(values: ParsedScriptArgs["values"], name: string): string[] {
  const value = values[name];
  if (Array.isArray(value)) return value;
  return typeof value === "string" ? [value] : [];
}

export function booleanOption(
  values: ParsedScriptArgs["values"],
  name: string,
  defaultValue = false
): boolean {
  const value = values[name];
  return typeof value === "boolean" ? value : defaultValue;
}
