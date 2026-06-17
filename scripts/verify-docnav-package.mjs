import {
  parseManifestArgs,
  resolvePackageManifestPath,
  validateReleasePackage,
} from "./tools/release-package/index.mjs";

const selection = parseManifestArgs(process.argv.slice(2));
const manifestPath = resolvePackageManifestPath(selection);
const { expectProducerKind, expectSourceDirty } = selection;

try {
  const result = validateReleasePackage(manifestPath, {
    expectProducerKind,
    expectSourceDirty
  });
  console.log("");
  console.log("Docnav Release Package Validation");
  console.log("Status: passed");
  console.log(`Version: ${result.manifest.version}`);
  console.log(`Target: ${result.manifest.target}`);
  console.log(`Package: ${result.packageDir}`);
  console.log(`Files: ${result.manifest.files.length + 2}`);
  console.log("");
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
