import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { FILE_SYSTEM } from "./config.ts";
import { errorMessage, isJsonValue } from "../types.ts";
import type { JsonValue } from "../types.ts";

export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

const ignoredDirs = new Set(FILE_SYSTEM.ignoredDirs);

export function toAbs(relPath: string): string {
  return path.join(root, relPath);
}

export function toRel(absPath: string): string {
  return path.relative(root, absPath).replaceAll(path.sep, "/");
}

export function walk(dir: string, predicate: (filePath: string) => boolean = () => true): string[] {
  const results: string[] = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      if (!ignoredDirs.has(entry.name)) {
        results.push(...walk(path.join(dir, entry.name), predicate));
      }
      continue;
    }

    const filePath = path.join(dir, entry.name);
    if (predicate(filePath)) {
      results.push(filePath);
    }
  }
  return results;
}

export function readJson(relPath: string): JsonValue {
  const source = fs.readFileSync(toAbs(relPath), "utf8");
  let parsed: unknown;
  try {
    parsed = JSON.parse(source);
  } catch (error: unknown) {
    throw new Error(`${relPath} JSON parse failed: ${errorMessage(error)}`, { cause: error });
  }
  assert(isJsonValue(parsed), `${relPath} must contain a JSON value`);
  return parsed;
}

export function listExampleJson(pattern: RegExp): string[] {
  return fs
    .readdirSync(toAbs(FILE_SYSTEM.examplesJsonDir))
    .filter((name) => pattern.test(name))
    .map((name) => `${FILE_SYSTEM.examplesJsonDir}/${name}`)
    .sort();
}

export function listSchemaJson() {
  return walk(toAbs(FILE_SYSTEM.schemasDir), (filePath) =>
    filePath.endsWith(FILE_SYSTEM.schemaExtension)
  )
    .map(toRel)
    .sort();
}

export function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

export function assertDeepEqual(actual: unknown, expected: unknown, message: string): void {
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {
    throw new Error(`${message}\nactual: ${actualJson}\nexpected: ${expectedJson}`);
  }
}

export function charLength(value: string): number {
  return [...value].length;
}
