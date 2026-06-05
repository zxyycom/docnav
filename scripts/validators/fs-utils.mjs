import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { FILE_SYSTEM } from "./config.mjs";

export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const ignoredDirs = new Set(FILE_SYSTEM.ignoredDirs);

export function toAbs(relPath) {
  return path.join(root, relPath);
}

export function toRel(absPath) {
  return path.relative(root, absPath).replaceAll(path.sep, "/");
}

export function walk(dir, predicate = () => true) {
  const results = [];
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

export function readJson(relPath) {
  return JSON.parse(fs.readFileSync(toAbs(relPath), "utf8"));
}

export function listExampleJson(pattern) {
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

export function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

export function assertDeepEqual(actual, expected, message) {
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {
    throw new Error(`${message}\nactual: ${actualJson}\nexpected: ${expectedJson}`);
  }
}

export function charLength(value) {
  return [...value].length;
}
