import type { ReleaseManifest } from "./config.ts";

const PACKAGE_METADATA_FILE_COUNT = 2;

type ReleasePackageSummary = {
  manifest: ReleaseManifest;
  packageDir: string;
  title: string;
};

export function printReleasePackageSummary({
  manifest,
  packageDir,
  title,
}: ReleasePackageSummary): void {
  console.log("");
  console.log(title);
  console.log("Status: passed");
  console.log(`Version: ${manifest.version}`);
  console.log(`Target: ${manifest.target}`);
  console.log(`Package: ${packageDir}`);
  console.log(`Files: ${manifest.files.length + PACKAGE_METADATA_FILE_COUNT}`);
  console.log("");
}
