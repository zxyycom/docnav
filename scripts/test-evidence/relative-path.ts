import fs from "node:fs";
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

export function resolveExistingWorkspacePath(
  workspaceRoot: string,
  sourcePath: string,
  label: string
): {
  absolutePath: string;
  stats: fs.Stats;
} {
  if (!isSafeRelativePosixPath(sourcePath)) {
    throw new Error(`${label} must be a safe relative POSIX path`);
  }
  const absoluteRoot = path.resolve(workspaceRoot);
  const absolutePath = path.resolve(absoluteRoot, ...sourcePath.split("/"));
  assertWithinWorkspace(absoluteRoot, absolutePath, label);
  return {
    absolutePath,
    stats: readPathComponents(absoluteRoot, sourcePath, label)
  };
}

function readPathComponents(
  absoluteRoot: string,
  sourcePath: string,
  label: string
): fs.Stats {
  let currentPath = absoluteRoot;
  const segments = sourcePath.split("/");
  for (const [index, segment] of segments.entries()) {
    currentPath = path.join(currentPath, segment);
    if (!fs.existsSync(currentPath)) {
      throw new Error(`${label} does not exist`);
    }
    const stats = fs.lstatSync(currentPath);
    if (stats.isSymbolicLink()) {
      const component = segments.slice(0, index + 1).join("/");
      throw new Error(
        `${label} path component must not be a symbolic link: ${component}`
      );
    }
    if (index < segments.length - 1 && !stats.isDirectory()) {
      const component = segments.slice(0, index + 1).join("/");
      throw new Error(`${label} path component must be a directory: ${component}`);
    }
    if (index === segments.length - 1) {
      return stats;
    }
  }
  throw new Error(`${label} does not exist`);
}

function assertWithinWorkspace(
  absoluteRoot: string,
  absolutePath: string,
  label: string
): void {
  const relativePath = path.relative(absoluteRoot, absolutePath);
  if (
    relativePath === ".." ||
    relativePath.startsWith(`..${path.sep}`) ||
    path.isAbsolute(relativePath)
  ) {
    throw new Error(`${label} must remain inside the current checkout`);
  }
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
