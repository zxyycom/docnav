import path from "node:path";

import { discoverBunEntities } from "./discovery/bun.ts";
import { discoverRustEntities } from "./discovery/rust.ts";
import { discoverSmokeEntities } from "./discovery/smoke.ts";
import {
  diagnostic,
  type DiscoveryResult,
  type TestEntity
} from "./model.ts";
import {
  loadSupportedRunnerProfile,
  workspaceRoot as supportedWorkspaceRoot,
  type SupportedRunnerProfile
} from "./profile.ts";

export async function discoverTestEntities(options: {
  workspaceRoot: string;
}): Promise<DiscoveryResult> {
  const workspaceRoot = path.resolve(options.workspaceRoot);
  const profileResolution = resolveDiscoveryProfile(workspaceRoot);
  if (profileResolution.failure) {
    return profileResolution.failure;
  }
  const profile = profileResolution.profile;
  const discovered = await discoverRunnerEntities(workspaceRoot, profile);
  const diagnostics = [
    ...discovered.diagnostics,
    ...duplicateEntityDiagnostics(discovered.entities)
  ];
  return {
    profile: {
      id: profile.id,
      version: profile.version
    },
    entities: discovered.entities,
    diagnostics
  };
}

type ProfileResolution =
  | {
      profile: SupportedRunnerProfile;
      failure?: never;
    }
  | {
      profile?: never;
      failure: DiscoveryResult;
    };

function resolveDiscoveryProfile(workspaceRoot: string): ProfileResolution {
  if (workspaceRoot !== supportedWorkspaceRoot) {
    return {
      failure: invalidProfileDiscovery(
        `native test discovery must use the current checkout ${supportedWorkspaceRoot}; received ${workspaceRoot}`,
        workspaceRoot
      )
    };
  }
  try {
    return {
      profile: loadSupportedRunnerProfile()
    };
  } catch (error) {
    return {
      failure: invalidProfileDiscovery(
        error instanceof Error ? error.message : String(error)
      )
    };
  }
}

async function discoverRunnerEntities(
  workspaceRoot: string,
  profile: SupportedRunnerProfile
): Promise<{
  entities: TestEntity[];
  diagnostics: DiscoveryResult["diagnostics"];
}> {
  const rust = await discoverRustEntities({
    workspaceRoot,
    profile
  });
  const bun = await discoverBunEntities({
    workspaceRoot,
    profile
  });
  const smoke = await discoverSmokeEntities({
    workspaceRoot,
    profile
  });
  return {
    entities: [
      ...rust.entities,
      ...bun.entities,
      ...smoke.entities
    ].sort(compareEntities),
    diagnostics: [
      ...rust.diagnostics,
      ...bun.diagnostics,
      ...smoke.diagnostics
    ]
  };
}

function duplicateEntityDiagnostics(
  entities: readonly TestEntity[]
): DiscoveryResult["diagnostics"] {
  const diagnostics: DiscoveryResult["diagnostics"] = [];
  for (let index = 1; index < entities.length; index += 1) {
    if (entities[index - 1]?.entityKey === entities[index]?.entityKey) {
      const entity = entities[index];
      diagnostics.push(diagnostic(
        "duplicate-entity",
        "runner",
        `multiple runner adapters produced entity key ${entity.entityKey}`,
        {
          runner: entity.runner,
          target: entity.target,
          selector: entity.selector,
          entityKey: entity.entityKey,
          path: entity.sourcePath
        }
      ));
    }
  }
  return diagnostics;
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
    entities: [],
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

function compareEntities(left: TestEntity, right: TestEntity): number {
  return left.entityKey < right.entityKey
    ? -1
    : left.entityKey > right.entityKey ? 1 : 0;
}
