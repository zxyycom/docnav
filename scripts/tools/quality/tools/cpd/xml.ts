import path from "node:path";
import { XMLParser } from "fast-xml-parser";

import type { DuplicateCodeFragment, DuplicateCodeLocation } from "../../schema.ts";
import { toSlashPath } from "../../../path/utils.ts";
import { errorMessage, isNonArrayRecord } from "../../../types.ts";
import type { CpdScanResult } from "./types.ts";

const cpdXmlParser = new XMLParser({
  attributeNamePrefix: "",
  ignoreAttributes: false,
  isArray: (tagName, _jPath, _isLeafNode, isAttribute) =>
    !isAttribute && (tagName === "duplication" || tagName === "file"),
  parseAttributeValue: false,
  parseTagValue: false,
  trimValues: true
});

/**
 * 解析 CPD XML 输出。
 *
 * CPD 格式：
 * ```xml
 * <?xml version="1.0" encoding="UTF-8"?>
 * <pmd-cpd>
 *   <duplication lines="10" tokens="50">
 *     <file path="/path/to/file1.rs" line="10" endline="20"/>
 *     <file path="/path/to/file2.rs" line="5" endline="15"/>
 *   </duplication>
 * </pmd-cpd>
 * ```
 */
export function parseCpdXml(xml: string, cwd: string): CpdScanResult {
  try {
    const fragments = parseCpdDuplications(xml)
      .map((duplication, index) => parseCpdFragment(duplication, cwd, index + 1))
      .sort((a, b) => b.tokenCount - a.tokenCount);

    return { ok: true, fragments };
  } catch (error: unknown) {
    return { ok: false, skipped: false, error: `Failed to parse CPD XML: ${errorMessage(error)}` };
  }
}

function parseCpdDuplications(xml: string): Record<string, unknown>[] {
  const parsed = cpdXmlParser.parse(xml) as unknown;
  const root = isNonArrayRecord(parsed) ? parsed["pmd-cpd"] : undefined;
  return isNonArrayRecord(root) ? toRecordArray(root.duplication) : [];
}

function parseCpdFragment(
  duplication: Record<string, unknown>,
  cwd: string,
  id: number
): DuplicateCodeFragment {
  const lines = parseIntegerAttribute(duplication, "lines");
  const tokens = parseIntegerAttribute(duplication, "tokens");

  return {
    id,
    tokenCount: tokens,
    lineCount: lines,
    locations: parseCpdLocations(duplication, cwd, lines),
    codeAreas: [],
    hitsChangedScope: false
  };
}

function parseCpdLocations(
  duplication: Record<string, unknown>,
  cwd: string,
  duplicationLines: number
): DuplicateCodeLocation[] {
  const locations = toRecordArray(duplication.file).map((file) =>
    parseCpdLocation(file, cwd, duplicationLines)
  );

  if (locations.length === 0) {
    throw new Error("CPD XML duplication must include at least one file location");
  }

  return locations;
}

function parseCpdLocation(
  file: Record<string, unknown>,
  cwd: string,
  duplicationLines: number
): DuplicateCodeLocation {
  const rawPath = stringAttribute(file, "path");
  const rawLine = stringAttribute(file, "line");
  if (!rawPath || !rawLine) {
    throw new Error("CPD XML file entry must include path and line attributes");
  }

  const startLine = parseIntegerAttribute(file, "line");
  return {
    path: normalizeCpdPath(rawPath, cwd),
    startLine,
    endLine: parseCpdEndLine(file, startLine, duplicationLines),
    codeArea: "unknown"
  };
}

function parseCpdEndLine(
  file: Record<string, unknown>,
  startLine: number,
  duplicationLines: number
): number {
  return stringAttribute(file, "endline") !== undefined
    ? parseIntegerAttribute(file, "endline")
    : startLine + duplicationLines;
}

function parseIntegerAttribute(attrs: Record<string, unknown>, name: string): number {
  const value = stringAttribute(attrs, name);
  if (value === undefined) {
    throw new Error(`CPD XML attribute "${name}" is required`);
  }
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed)) {
    throw new Error(`CPD XML attribute "${name}" must be an integer`);
  }
  return parsed;
}

function stringAttribute(attrs: Record<string, unknown>, name: string): string | undefined {
  const value = attrs[name];
  return typeof value === "string" ? value : undefined;
}

function normalizeCpdPath(filePath: string, cwd: string): string {
  if (!path.isAbsolute(filePath)) {
    return toSlashPath(filePath);
  }

  const relativePath = path.relative(cwd, filePath);
  if (relativePath === "") {
    return ".";
  }
  if (!relativePath.startsWith("..") && !path.isAbsolute(relativePath)) {
    return toSlashPath(relativePath);
  }
  return toSlashPath(filePath);
}

function toRecordArray(value: unknown): Record<string, unknown>[] {
  if (value === undefined) return [];
  const values = Array.isArray(value) ? value : [value];
  return values.filter(isNonArrayRecord);
}
