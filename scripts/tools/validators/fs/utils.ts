import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { FILE_SYSTEM } from "../config.ts";
import { readTextFile, walkFiles } from "../../fs.ts";
import { toSlashPath } from "../../path/utils.ts";
import { parseJsonValue } from "../../types.ts";
import type { JsonValue } from "../../types.ts";

export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");

const ignoredDirs = new Set(FILE_SYSTEM.ignoredDirs);

export function toAbs(relPath: string): string {
  return path.join(root, relPath);
}

export function toRel(absPath: string): string {
  return toSlashPath(path.relative(root, absPath));
}

export function walk(dir: string, predicate: (filePath: string) => boolean = () => true): string[] {
  return walkFiles(dir, { ignoredDirs })
    .map((relPath) => path.join(dir, relPath))
    .filter(predicate);
}

export function readJson(relPath: string): JsonValue {
  return parseJsonValue(readTextFile(toAbs(relPath)), `${relPath} JSON`);
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
