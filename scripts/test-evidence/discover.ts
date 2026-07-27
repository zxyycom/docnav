import path from "node:path";

import { canonicalJson } from "./fingerprint.ts";
import { discoverBunEntries } from "./discovery/bun.ts";
import { discoverRustEntries } from "./discovery/rust.ts";
import { discoverSmokeEntries } from "./discovery/smoke.ts";
import {
  diagnostic,
  type DiscoveryResult,
  type NativeTestEntry
} from "./model.ts";
import {
  loadSupportedRunnerProfile,
  workspaceRoot as supportedWorkspaceRoot,
  type SupportedRunnerProfile
} from "./profile.ts";

export async function discoverNativeTestEntries(options: {
  workspaceRoot: string;
}): Promise<DiscoveryResult> {
  const workspaceRoot = path.resolve(options.workspaceRoot);
  if (workspaceRoot !== supportedWorkspaceRoot) {
    return invalidProfileDiscovery(
      `native test discovery must use the current checkout ${supportedWorkspaceRoot}; received ${workspaceRoot}`,
      workspaceRoot
    );
  }

  let profile: SupportedRunnerProfile;
  try {
    profile = loadSupportedRunnerProfile();
  } catch (error) {
    return invalidProfileDiscovery(
      error instanceof Error ? error.message : String(error)
    );
  }

  const rust = await discoverRustEntries({
    workspaceRoot,
    profile
  });
  const bun = await discoverBunEntries({
    workspaceRoot,
    profile
  });
  const smoke = await discoverSmokeEntries({
    workspaceRoot,
    profile
  });
  const diagnostics = [
    ...rust.diagnostics,
    ...bun.diagnostics,
    ...smoke.diagnostics
  ];
  const entries = [
    ...rust.entries,
    ...bun.entries,
    ...smoke.entries
  ].sort(compareEntries);

  for (let index = 1; index < entries.length; index += 1) {
    if (entries[index - 1]?.entryKey === entries[index]?.entryKey) {
      const entry = entries[index];
      diagnostics.push(diagnostic(
        "duplicate-entry",
        "inventory",
        `multiple runner adapters produced entryKey ${entry.entryKey}`,
        {
          runner: entry.runner,
          target: entry.target,
          selector: entry.selector,
          entryKey: entry.entryKey,
          path: entry.sourcePath
        }
      ));
    }
  }

  return {
    profile: {
      id: profile.id,
      version: profile.version
    },
    entries,
    diagnostics
  };
}

function invalidProfileDiscovery(
  message: string,
  sourcePath?: string
): DiscoveryResult {
  return {
    profile: {
      id: "invalid-profile",
      version: 1
    },
    entries: [],
    diagnostics: [
      diagnostic(
        "runner-profile-invalid",
        "profile",
        message,
        sourcePath === undefined ? {} : { path: sourcePath }
      )
    ]
  };
}

function compareEntries(left: NativeTestEntry, right: NativeTestEntry): number {
  const leftKey = canonicalJson(left.entryKey);
  const rightKey = canonicalJson(right.entryKey);
  return leftKey < rightKey ? -1 : leftKey > rightKey ? 1 : 0;
}
