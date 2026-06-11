import path from "node:path";

import {
  parseManifestArgs,
  root,
  runNodeScript,
  validateReleasePackage
} from "./release-package.mjs";

const { manifestPath, expectProducerKind, expectSourceDirty } = parseManifestArgs(process.argv.slice(2));
const { manifest, packageDir } = validateReleasePackage(manifestPath, {
  expectProducerKind,
  expectSourceDirty
});

const env = {
  ...process.env,
  DOCNAV_BIN: path.join(packageDir, binaryName("docnav", manifest.target)),
  DOCNAV_MARKDOWN_BIN: path.join(packageDir, binaryName("docnav-markdown", manifest.target))
};

try {
  console.log("");
  console.log("Docnav Release Package Smoke");
  console.log("Status: running");
  console.log(`Package: ${packageDir}`);
  console.log("");

  runNodeScript(path.join(root, "scripts", "docnav-markdown-cli-smoke.mjs"), [], { env });
  runNodeScript(path.join(root, "scripts", "docnav-core-cli-smoke.mjs"), [], { env });

  console.log("");
  console.log("Docnav Release Package Smoke");
  console.log("Status: passed");
  console.log(`Package: ${packageDir}`);
  console.log("");
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}

function binaryName(binName, target) {
  return target.includes("windows") ? `${binName}.exe` : binName;
}
