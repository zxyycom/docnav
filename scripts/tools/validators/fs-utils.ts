import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { FILE_SYSTEM } from "./config.ts";

export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

const ignoredDirs = new Set(FILE_SYSTEM.ignoredDirs);

export function toAbs(relPath: any) {
  return path.join(root, relPath);
}

export function toRel(absPath: any) {
  return path.relative(root, absPath).replaceAll(path.sep, "/");
}

export function walk(dir: any, predicate: (filePath: string) => boolean = () => true) {
  const results: any[] = [];
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

export function readJson(relPath: any) {
  const source = fs.readFileSync(toAbs(relPath), "utf8");
  try {
    return JSON.parse(source);
  } catch (error: any) {
    throw new Error(`${relPath} JSON parse failed: ${error.message}`);
  }
}

export function listExampleJson(pattern: any) {
  return fs
    .readdirSync(toAbs(FILE_SYSTEM.examplesJsonDir))
    .filter((name) => pattern.test(name))
    .map((name) => `${FILE_SYSTEM.examplesJsonDir}/${name}`)
    .sort();
}

export function listSchemaJson() {
  return walk(toAbs(FILE_SYSTEM.schemasDir), (filePath: any) =>
    filePath.endsWith(FILE_SYSTEM.schemaExtension)
  )
    .map(toRel)
    .sort();
}

export function assert(condition: any, message: any) {
  if (!condition) {
    throw new Error(message);
  }
}

export function assertDeepEqual(actual: any, expected: any, message: any) {
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {
    throw new Error(`${message}\nactual: ${actualJson}\nexpected: ${expectedJson}`);
  }
}

export function charLength(value: any) {
  return [...value].length;
}
