import path from "node:path";

import {
  parseManifestArgs,
  resolvePackageManifestPath,
  root,
  runScript,
  validateReleasePackage,
} from "../tools/release-package/index.ts";

const selection = parseManifestArgs(process.argv.slice(2));
const manifestPath = resolvePackageManifestPath(selection);
const { expectProducerKind, expectSourceDirty } = selection;
const { manifest, packageDir } = validateReleasePackage(manifestPath, {
  expectProducerKind,
  expectSourceDirty,
});
const coreEntries = manifest.files.filter(
  (entry) => entry.component === "core",
);
const coreEntry = coreEntries[0];
if (coreEntries.length !== 1 || !coreEntry) {
  throw new Error(
    `validated manifest must contain exactly one core executable entry; found ${coreEntries.length}`,
  );
}

const resolvedPackageDir = path.resolve(packageDir);
const docnavBinaryPath = path.resolve(resolvedPackageDir, coreEntry.path);
if (path.dirname(docnavBinaryPath) !== resolvedPackageDir) {
  throw new Error(
    `validated core executable must resolve directly inside package directory: ${coreEntry.path}`,
  );
}

const env = {
  ...process.env,
  DOCNAV_BIN: docnavBinaryPath,
  DOCNAV_SMOKE_PROFILE: "release-package",
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
