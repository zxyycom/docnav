import { buildReleasePackage, parseOptionalTarget, resolveHostTarget } from "./tools/release-package/index.mjs";

const target = parseOptionalTarget(process.argv.slice(2)) ?? resolveHostTarget();

try {
  const result = buildReleasePackage(target);
  console.log("");
  console.log("Docnav Release Package");
  console.log("Status: passed");
  console.log(`Version: ${result.manifest.version}`);
  console.log(`Target: ${result.manifest.target}`);
  console.log(`Package: ${result.packageDir}`);
  console.log(`Files: ${result.fileCount + 2}`);
  console.log("");
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
