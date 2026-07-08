import path from "node:path";

import {
  expectedBinaryName,
  releaseComponents,
} from "./config.ts";
import { readJsonFile } from "../foundation/src/fs.ts";
import { sha256File } from "./io.ts";
import { validateManifestFiles } from "./validation/manifest/files.ts";
import {
  validateManifestLocation,
  validateManifestMetadata,
} from "./validation/manifest/metadata.ts";
import type { ManifestValidationOptions } from "./validation/manifest/metadata.ts";
import {
  validateChecksums,
  validatePackageDirectory,
  validatePackagedBinaries,
} from "./validation/package.ts";

export type { ManifestValidationOptions } from "./validation/manifest/metadata.ts";

export function validateReleasePackage(manifestPath: string, options: ManifestValidationOptions = {}) {
  const resolvedManifestPath = path.resolve(manifestPath);
  const packageDir = path.dirname(resolvedManifestPath);
  const manifest = readJsonFile(resolvedManifestPath);

  validateManifestMetadata(manifest, options);
  validateManifestLocation(resolvedManifestPath, packageDir, manifest);

  const expectedFileNames = releaseComponents.map((component) =>
    expectedBinaryName(component.binName, manifest.target),
  );
  const manifestFilesByPath = validateManifestFiles(
    manifest.files,
    expectedFileNames,
  );

  validatePackageDirectory(packageDir, expectedFileNames);
  validatePackagedBinaries(packageDir, expectedFileNames, manifestFilesByPath);

  const manifestHash = sha256File(resolvedManifestPath);
  validateChecksums(
    packageDir,
    resolvedManifestPath,
    expectedFileNames,
    manifestFilesByPath,
    manifestHash,
  );

  return {
    manifestPath: resolvedManifestPath,
    packageDir,
    manifest,
    manifestHash,
    fileEntries: expectedFileNames.map((fileName) => ({
      ...manifestFilesByPath.get(fileName),
      path: fileName,
    })),
  };
}
