import { resolvePackageLayout } from "./config.mjs";
import { resolveHostTarget, resolveWorkspaceVersion } from "./environment.mjs";

export function resolvePackageManifestPath({ manifestPath, target }) {
  if (manifestPath) {
    return manifestPath;
  }

  const selectedTarget = target ?? resolveHostTarget();
  const version = resolveWorkspaceVersion();
  return resolvePackageLayout(version, selectedTarget).manifestPath;
}
