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
  const diagnostics = [
    ...rust.diagnostics,
    ...bun.diagnostics,
    ...smoke.diagnostics
  ];
  const entities = [
    ...rust.entities,
    ...bun.entities,
    ...smoke.entities
  ].sort(compareEntities);

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

  return {
    profile: {
      id: profile.id,
      version: profile.version
    },
    entities,
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
