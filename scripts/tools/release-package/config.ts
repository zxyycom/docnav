import path from "node:path";
import { fileURLToPath } from "node:url";

export const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
  "..",
);
export const artifactsRoot = path.join(root, "artifacts", "docnav");

export type ReleaseComponent =
  | Readonly<{
    component: "core";
    packageName: string;
    binName: string;
  }>
  | Readonly<{
    component: "adapter";
    adapterId: string;
    packageName: string;
    binName: string;
  }>;

export type PackageLayout = {
  version: string;
  target: string;
  releaseRoot: string;
  packageDir: string;
  manifestPath: string;
  checksumsPath: string;
};

export type ReleaseProducer =
  | {
    kind: "local";
    workflow: null;
    run_id: null;
    run_attempt: null;
  }
  | {
    kind: "github-actions";
    workflow: string;
    run_id: number;
    run_attempt: number;
  };

export type ReleaseManifestFile = {
  path: string;
  component: "adapter" | "core";
  adapter_id?: string;
  size_bytes: number;
  sha256: string;
};

export type ReleaseManifest = {
  schema_version: 1;
  product: "docnav";
  version: string;
  target: string;
  generated_at: string;
  git_commit: string;
  source_dirty: boolean;
  producer: ReleaseProducer;
  files: ReleaseManifestFile[];
};

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
]) satisfies readonly ReleaseComponent[];

export function resolvePackageLayout(version: string, target: string): PackageLayout {
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

export function resolveBinaryDestPath(packageDir: string, executablePath: string): string {
  return path.join(packageDir, path.basename(executablePath));
}

export function expectedBinaryName(binName: string, target: string): string {
  return target.includes("windows") ? `${binName}.exe` : binName;
}

export function compareStrings(left: string, right: string): number {
  if (left < right) {
    return -1;
  }
  if (left > right) {
    return 1;
  }
  return 0;
}
