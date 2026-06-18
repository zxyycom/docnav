import { resolvePackageLayout } from "./config.ts";
import { resolveHostTarget, resolveWorkspaceVersion } from "./environment.ts";
import type { ManifestArgs } from "./args.ts";

export function resolvePackageManifestPath({ manifestPath, target }: Pick<ManifestArgs, "manifestPath" | "target">): string {
  if (manifestPath) {
    return manifestPath;
  }

  const selectedTarget = target ?? resolveHostTarget();
  const version = resolveWorkspaceVersion();
  return resolvePackageLayout(version, selectedTarget).manifestPath;
}
