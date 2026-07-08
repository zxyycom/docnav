import path from "node:path";

import { releaseComponents } from "../../config.ts";
import type { ReleaseManifest, ReleaseProducer } from "../../config.ts";
import { isRecord } from "../../../foundation/src/type-guards.ts";
import {
  assert,
  assertNonEmptyString,
} from "../assertions.ts";
import { validateManifestFile } from "./files.ts";
import { validateProducer } from "./producer.ts";

export type ManifestValidationOptions = {
  expectProducerKind?: ReleaseProducer["kind"] | null;
  expectSourceDirty?: boolean | null;
};

export function validateManifestLocation(
  manifestPath: string,
  packageDir: string,
  manifest: ReleaseManifest,
): void {
  assert(
    path.basename(manifestPath) === "manifest.json",
    "manifest path must end with manifest.json",
  );
  assert(
    path.basename(packageDir) === "package",
    "manifest must live in a package/ directory",
  );
  assert(
    path.basename(path.dirname(packageDir)) === manifest.target,
    "package target directory must match manifest target",
  );
  assert(
    path.basename(path.dirname(path.dirname(packageDir))) ===
      `v${manifest.version}`,
    "package version directory must match manifest version",
  );
  assert(
    path.basename(path.dirname(path.dirname(path.dirname(packageDir)))) ===
      "docnav",
    "package root must be artifacts/docnav",
  );
}

export function validateManifestMetadata(
  manifest: unknown,
  options: ManifestValidationOptions,
): asserts manifest is ReleaseManifest {
  assert(isRecord(manifest), "manifest root must be an object");
  assert(manifest.schema_version === 1, "manifest.schema_version must be 1");
  assert(manifest.product === "docnav", "manifest.product must be docnav");
  assertNonEmptyString(manifest.version, "manifest.version");
  assertNonEmptyString(manifest.target, "manifest.target");
  assertNonEmptyString(manifest.generated_at, "manifest.generated_at");
  assertNonEmptyString(manifest.git_commit, "manifest.git_commit");
  assert(
    typeof manifest.source_dirty === "boolean",
    "manifest.source_dirty must be a boolean",
  );
  assert(Array.isArray(manifest.files), "manifest.files must be an array");
  assert(
    manifest.files.length === releaseComponents.length,
    "manifest.files must list all release binaries",
  );

  const producer = manifest.producer;
  validateProducer(producer);
  for (const entry of manifest.files) {
    validateManifestFile(entry);
  }

  if (options.expectProducerKind) {
    assert(
      producer.kind === options.expectProducerKind,
      `manifest.producer.kind must be ${options.expectProducerKind}`,
    );
  }
  if (
    options.expectSourceDirty !== null &&
    options.expectSourceDirty !== undefined
  ) {
    assert(
      manifest.source_dirty === options.expectSourceDirty,
      `manifest.source_dirty must be ${options.expectSourceDirty}`,
    );
  }
}
