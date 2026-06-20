import {
  buildReleasePackage,
  parseOptionalTarget,
  printReleasePackageSummary,
  resolveHostTarget
} from "../tools/release-package/index.ts";

const target = parseOptionalTarget(process.argv.slice(2)) ?? resolveHostTarget();

try {
  const result = buildReleasePackage(target);
  printReleasePackageSummary({
    title: "Docnav Release Package",
    manifest: result.manifest,
    packageDir: result.packageDir
  });
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
