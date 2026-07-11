import path from "node:path";

import {
  expectedBinaryName,
  parseManifestArgs,
  resolvePackageManifestPath,
  root,
  runScript,
  validateReleasePackage,
} from "../tools/release-package/index.ts";

// @case BB-RELEASE-PACKAGE-001
const selection = parseManifestArgs(process.argv.slice(2));
const manifestPath = resolvePackageManifestPath(selection);
const { expectProducerKind, expectSourceDirty } = selection;
const { manifest, packageDir } = validateReleasePackage(manifestPath, {
  expectProducerKind,
  expectSourceDirty,
});

const env = {
  ...process.env,
  DOCNAV_BIN: path.join(
    packageDir,
    expectedBinaryName("docnav", manifest.target),
  ),
};

try {
  console.log("");
  console.log("Docnav Release Package Smoke");
  console.log("Status: running");
  console.log(`Package: ${packageDir}`);
  console.log("");

  runScript(
    path.join(root, "test", "docnav-core-smoke.ts"),
    [],
    { env },
  );

  console.log("");
  console.log("Docnav Release Package Smoke");
  console.log("Status: passed");
  console.log(`Package: ${packageDir}`);
  console.log("");
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
