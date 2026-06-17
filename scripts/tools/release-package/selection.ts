import { resolvePackageLayout } from "./config.ts";
import { resolveHostTarget, resolveWorkspaceVersion } from "./environment.ts";

export function resolvePackageManifestPath({ manifestPath, target }: any) {
  if (manifestPath) {
    return manifestPath;
  }

  const selectedTarget = target ?? resolveHostTarget();
  const version = resolveWorkspaceVersion();
  return resolvePackageLayout(version, selectedTarget).manifestPath;
}
