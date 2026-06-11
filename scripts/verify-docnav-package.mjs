import { parseManifestArgs, validateReleasePackage } from "./release-package.mjs";

const { manifestPath, expectProducerKind, expectSourceDirty } = parseManifestArgs(process.argv.slice(2));

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
