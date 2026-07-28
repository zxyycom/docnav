import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  type BunTestSurface,
  isSafeRelativeGlob
} from "./discovery/bun-files.ts";
import { isSafeRelativePosixPath } from "./relative-path.ts";

const SLUG_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;
export const SUPPORTED_SMOKE_FACTORY = "test/smoke/core/profile.ts";

export type SupportedRunnerProfile = {
  schemaVersion: 2;
  id: string;
  version: number;
  cargo: {
    sourceRoots: string[];
    targetKinds: Array<"bin" | "lib" | "test">;
    doctests: "block-until-supported";
  };
  bun: BunTestSurface;
  smoke: {
    id: string;
    factory: string;
    sourceRoots: string[];
  };
};

export const workspaceRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  ".."
);

export const profilePath = path.join(
  workspaceRoot,
  "scripts",
  "test-evidence",
  "supported-runner-profile.json"
);

export function loadSupportedRunnerProfile(
  sourcePath = profilePath
): SupportedRunnerProfile {
  const value: unknown = JSON.parse(fs.readFileSync(sourcePath, "utf8"));
  if (!isRecord(value) || !hasExactKeys(value, [
    "schemaVersion",
    "id",
    "version",
    "cargo",
    "bun",
    "smoke"
  ])) {
    throw new Error("supported runner profile has an invalid root shape");
  }
  if (
    value.schemaVersion !== 2 ||
    typeof value.id !== "string" ||
    !SLUG_PATTERN.test(value.id) ||
    !Number.isInteger(value.version) ||
    Number(value.version) < 1
  ) {
    throw new Error("supported runner profile identity is invalid");
  }
  const cargo = parseCargoProfile(value.cargo);
  const bun = parseBunProfile(value.bun);
  const smoke = parseSmokeProfile(value.smoke);
  return {
    schemaVersion: 2,
    id: value.id,
    version: Number(value.version),
    cargo,
    bun,
    smoke
  };
}

function parseCargoProfile(value: unknown): SupportedRunnerProfile["cargo"] {
  if (!isRecord(value) || !hasExactKeys(value, [
    "sourceRoots",
    "targetKinds",
    "doctests"
  ])) {
    throw new Error("supported runner Cargo profile is invalid");
  }
  const sourceRoots = relativePathList(value.sourceRoots, "Cargo sourceRoots");
  const targetKinds = sortedStringList(value.targetKinds, "Cargo targetKinds");
  if (
    targetKinds.some((kind) => !["bin", "lib", "test"].includes(kind)) ||
    value.doctests !== "block-until-supported"
  ) {
    throw new Error("supported runner Cargo policy is invalid");
  }
  return {
    sourceRoots,
    targetKinds: targetKinds as Array<"bin" | "lib" | "test">,
    doctests: "block-until-supported"
  };
}

function parseBunProfile(value: unknown): SupportedRunnerProfile["bun"] {
  if (!isRecord(value) || !hasExactKeys(value, [
    "sourceRoots",
    "include",
    "ignore",
    "supplementalFiles"
  ])) {
    throw new Error("supported runner Bun profile is invalid");
  }
  const sourceRoots = relativePathList(value.sourceRoots, "Bun sourceRoots");
  const include = globList(value.include, "Bun include");
  const ignore = globList(value.ignore, "Bun ignore", { allowEmpty: true });
  const supplementalFiles = relativePathList(
    value.supplementalFiles,
    "Bun supplementalFiles",
    { allowEmpty: true }
  );
  return {
    sourceRoots,
    include,
    ignore,
    supplementalFiles
  };
}

function globList(
  value: unknown,
  label: string,
  options: { allowEmpty?: boolean } = {}
): string[] {
  const items = sortedStringList(value, label, options);
  if (items.some((item) => !isSafeRelativeGlob(item))) {
    throw new Error(`${label} must contain positive relative POSIX globs`);
  }
  return items;
}

function parseSmokeProfile(value: unknown): SupportedRunnerProfile["smoke"] {
  if (!isRecord(value) || !hasExactKeys(value, [
    "id",
    "factory",
    "sourceRoots"
  ])) {
    throw new Error("supported runner smoke profile is invalid");
  }
  if (
    typeof value.id !== "string" ||
    !SLUG_PATTERN.test(value.id) ||
    value.factory !== SUPPORTED_SMOKE_FACTORY
  ) {
    throw new Error("supported runner smoke identity is invalid");
  }
  return {
    id: value.id,
    factory: SUPPORTED_SMOKE_FACTORY,
    sourceRoots: relativePathList(value.sourceRoots, "smoke sourceRoots")
  };
}

function relativePathList(
  value: unknown,
  label: string,
  options: { allowEmpty?: boolean } = {}
): string[] {
  const items = sortedStringList(value, label, options);
  if (items.some((item) => !isSafeRelativePosixPath(item))) {
    throw new Error(`${label} must contain safe relative POSIX paths`);
  }
  return items;
}

function sortedStringList(
  value: unknown,
  label: string,
  options: { allowEmpty?: boolean } = {}
): string[] {
  if (
    !isUnknownArray(value) ||
    (!options.allowEmpty && value.length === 0)
  ) {
    throw new Error(
      `${label} must be ${options.allowEmpty ? "a" : "a non-empty"} string array`
    );
  }
  const items = value.map((item) => {
    if (
      typeof item !== "string" ||
      item.length === 0 ||
      item !== item.trim()
    ) {
      throw new Error(`${label} must be a non-empty string array`);
    }
    return item;
  });
  if (
    new Set(items).size !== items.length ||
    items.some((item, index) => index > 0 && items[index - 1] >= item)
  ) {
    throw new Error(`${label} must be uniquely sorted`);
  }
  return items;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isUnknownArray(value: unknown): value is unknown[] {
  return Array.isArray(value);
}

function hasExactKeys(
  value: Record<string, unknown>,
  keys: readonly string[]
): boolean {
  return JSON.stringify(Object.keys(value).sort()) === JSON.stringify([...keys].sort());
}
