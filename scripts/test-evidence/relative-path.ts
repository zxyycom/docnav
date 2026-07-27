import path from "node:path";

export function isSafeRelativePosixPath(value: unknown): value is string {
  if (typeof value !== "string") {
    return false;
  }
  return (
    isTrimmedNonEmpty(value) &&
    !path.posix.isAbsolute(value) &&
    !hasUnsafeSegment(value) &&
    path.posix.normalize(value) === value
  );
}

function isTrimmedNonEmpty(value: string): boolean {
  return value.length > 0 && value === value.trim();
}

function hasUnsafeSegment(value: string): boolean {
  return (
    value.includes("\\") ||
    value.includes("\0") ||
    value === "." ||
    value === ".." ||
    value.startsWith("../") ||
    value.endsWith("/")
  );
}
