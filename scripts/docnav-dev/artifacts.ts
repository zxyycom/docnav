import fs from "node:fs";
import path from "node:path";

import {
  assertNoSymlinkPathSegments,
  assertStrictDescendantPath
} from "../tools/foundation/src/fs.ts";

export const DEV_BIN_ARTIFACT_DIR = ".cache/docnav/verify";
export const DEV_BIN_COPY_DIR = `${DEV_BIN_ARTIFACT_DIR}/dev-bins`;
export const DEV_BIN_ENV_FILE = `${DEV_BIN_ARTIFACT_DIR}/dev-bins.json`;

export type DevBinArtifactPaths = Readonly<{
  artifactRoot: string;
  copyRoot: string;
  envFile: string;
  workspaceRoot: string;
}>;

export function resolveDevBinArtifactPaths(workspaceRoot: string): DevBinArtifactPaths {
  const resolvedWorkspaceRoot = fs.realpathSync(workspaceRoot);
  const paths = {
    artifactRoot: path.resolve(resolvedWorkspaceRoot, DEV_BIN_ARTIFACT_DIR),
    copyRoot: path.resolve(resolvedWorkspaceRoot, DEV_BIN_COPY_DIR),
    envFile: path.resolve(resolvedWorkspaceRoot, DEV_BIN_ENV_FILE),
    workspaceRoot: resolvedWorkspaceRoot
  };

  assertStrictDescendantPath(
    resolvedWorkspaceRoot,
    paths.artifactRoot,
    "development binary artifact root",
    "workspace root"
  );
  assertManagedDevBinArtifactPaths(paths);
  return paths;
}

export function assertManagedDevBinArtifactPaths(paths: DevBinArtifactPaths): void {
  assertStrictDescendantPath(
    paths.artifactRoot,
    paths.copyRoot,
    "development binary copy root",
    "artifact root"
  );
  assertStrictDescendantPath(
    paths.artifactRoot,
    paths.envFile,
    "development binary environment file",
    "artifact root"
  );
  assertNoSymlinkPathSegments(
    paths.workspaceRoot,
    paths.copyRoot,
    "development binary copy root"
  );
  assertNoSymlinkPathSegments(
    paths.workspaceRoot,
    paths.envFile,
    "development binary environment file"
  );
}
