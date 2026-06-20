import fs from "node:fs";

import { FILE_SYSTEM } from "../config.ts";
import { readTextFile } from "../../fs.ts";
import { parseJsonValue } from "../../json/value.ts";
import type { JsonValue } from "../../json/value.ts";
import { walk } from "../repo/files.ts";
import { toAbs, toRel } from "../repo/paths.ts";

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

export function listSchemaJson(): string[] {
  return walk(toAbs(FILE_SYSTEM.schemasDir), (filePath) =>
    filePath.endsWith(FILE_SYSTEM.schemaExtension)
  )
    .map(toRel)
    .sort();
}
