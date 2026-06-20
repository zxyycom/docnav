import fs from "node:fs";
import path from "node:path";

import {
  compareStrings,
  releaseComponents,
  resolveBinaryDestPath,
  resolvePackageLayout,
} from "./config.ts";
import type { PackageLayout, ReleaseManifest, ReleaseManifestFile } from "./config.ts";
import {
  buildReleaseBinary,
  getGitCommit,
  isSourceDirty,
  resolveHostTarget,
  resolveProducerMetadata,
  resolveWorkspaceVersion,
} from "./environment.ts";
import { writeJsonFile, writeTextFile } from "../fs.ts";
import { copyExecutable, sha256File } from "./io.ts";
import { toSlashPath } from "../path.ts";
import { validateReleasePackage } from "./validation.ts";

export type ReleasePackageBuildResult = PackageLayout & {
  manifest: ReleaseManifest;
  fileCount: number;
};

export function buildReleasePackage(target = resolveHostTarget()): ReleasePackageBuildResult {
  const version = resolveWorkspaceVersion();
  const layout = resolvePackageLayout(version, target);

  fs.rmSync(layout.releaseRoot, { recursive: true, force: true });
  fs.mkdirSync(layout.packageDir, { recursive: true });

  try {
    const files = buildPackageFiles(layout.packageDir, target);
    const manifest = createManifest(version, target, files);

    writeJsonFile(layout.manifestPath, manifest);
    writeChecksums(layout, manifest);
    validateReleasePackage(layout.manifestPath);

    return {
      ...layout,
      manifest,
      fileCount: manifest.files.length,
    };
  } catch (error) {
    fs.rmSync(layout.releaseRoot, { recursive: true, force: true });
    throw error;
  }
}

function buildPackageFiles(packageDir: string, target: string): ReleaseManifestFile[] {
  return releaseComponents.map((component) => {
    const executablePath = buildReleaseBinary(
      component.packageName,
      component.binName,
      target,
    );
    const destPath = resolveBinaryDestPath(packageDir, executablePath);
    copyExecutable(executablePath, destPath);

    return {
      path: toSlashPath(path.basename(destPath)),
      component: component.component,
      ...(component.component === "adapter" ? { adapter_id: component.adapterId } : {}),
      size_bytes: fs.statSync(destPath).size,
      sha256: sha256File(destPath),
    };
  });
}

function createManifest(version: string, target: string, files: ReleaseManifestFile[]): ReleaseManifest {
  return {
    schema_version: 1,
    product: "docnav",
    version,
    target,
    generated_at: new Date().toISOString(),
    git_commit: getGitCommit(),
    source_dirty: isSourceDirty(),
    producer: resolveProducerMetadata(),
    files: files.sort((left, right) => compareStrings(left.path, right.path)),
  };
}

function writeChecksums(layout: PackageLayout, manifest: ReleaseManifest): void {
  const checksumEntries = [
    ...manifest.files.map((entry) => [entry.path, entry.sha256]),
    ["manifest.json", sha256File(layout.manifestPath)],
  ];
  checksumEntries.sort((left, right) => compareStrings(left[0], right[0]));

  const content = checksumEntries
    .map(([fileName, hash]) => `${hash}  ${fileName}`)
    .join("\n");
  writeTextFile(layout.checksumsPath, `${content}\n`);
}
