import fs from "node:fs";

import {
  parseOptionalTarget,
  resolveHostTarget,
  resolvePackageLayout,
  resolveWorkspaceVersion
} from "./tools/release-package/index.mjs";

const target = parseOptionalTarget(process.argv.slice(2)) ?? resolveHostTarget();
const version = resolveWorkspaceVersion();
const layout = resolvePackageLayout(version, target);
const info = {
  version,
  target,
  packageDir: layout.packageDir,
  manifestPath: layout.manifestPath,
  checksumsPath: layout.checksumsPath
};

if (process.env.GITHUB_OUTPUT) {
  fs.appendFileSync(
    process.env.GITHUB_OUTPUT,
    [
      `version=${info.version}`,
      `target=${info.target}`,
      `package_dir=${info.packageDir}`,
      `manifest_path=${info.manifestPath}`,
      `checksums_path=${info.checksumsPath}`,
      ""
    ].join("\n"),
    "utf8"
  );
}

console.log("");
console.log("Docnav Release Package Info");
console.log(`Version: ${info.version}`);
console.log(`Target: ${info.target}`);
console.log(`Package: ${info.packageDir}`);
console.log(`Manifest: ${info.manifestPath}`);
console.log("");
