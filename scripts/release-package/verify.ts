import {
  parseManifestArgs,
  printReleasePackageSummary,
  resolvePackageManifestPath,
  validateReleasePackage,
} from "../tools/release-package/index.ts";

const selection = parseManifestArgs(process.argv.slice(2));
const manifestPath = resolvePackageManifestPath(selection);
const { expectProducerKind, expectSourceDirty } = selection;

try {
  const result = validateReleasePackage(manifestPath, {
    expectProducerKind,
    expectSourceDirty
  });
  printReleasePackageSummary({
    title: "Docnav Release Package Validation",
    manifest: result.manifest,
    packageDir: result.packageDir
  });
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
