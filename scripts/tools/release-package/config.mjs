import path from "node:path";
import { fileURLToPath } from "node:url";

export const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
  "..",
);
export const artifactsRoot = path.join(root, "artifacts", "docnav");

// 发布组件集合由 cli-artifact-layout change 明确定义，不从 Cargo workspace 自动推断。
export const releaseComponents = Object.freeze([
  Object.freeze({
    component: "core",
    packageName: "docnav",
    binName: "docnav",
  }),
  Object.freeze({
    component: "adapter",
    adapterId: "docnav-markdown",
    packageName: "docnav-markdown",
    binName: "docnav-markdown",
  }),
]);

export function resolvePackageLayout(version, target) {
  const packageDir = path.join(artifactsRoot, `v${version}`, target, "package");
  return {
    version,
    target,
    releaseRoot: path.dirname(packageDir),
    packageDir,
    manifestPath: path.join(packageDir, "manifest.json"),
    checksumsPath: path.join(packageDir, "SHA256SUMS.txt"),
  };
}

export function resolveBinaryDestPath(packageDir, executablePath) {
  return path.join(packageDir, path.basename(executablePath));
}

export function expectedBinaryName(binName, target) {
  return target.includes("windows") ? `${binName}.exe` : binName;
}

export function compareStrings(left, right) {
  if (left < right) {
    return -1;
  }
  if (left > right) {
    return 1;
  }
  return 0;
}
