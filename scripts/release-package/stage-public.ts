import {
  parseManifestArgs,
  stagePublicFiles,
} from "../tools/release-package/index.ts";

try {
  const selection = parseManifestArgs(process.argv.slice(2));
  if (!selection.manifestPath) {
    throw new Error("public staging requires --manifest <path>");
  }

  const result = stagePublicFiles(selection.manifestPath, {
    expectProducerKind: selection.expectProducerKind,
    expectSourceDirty: selection.expectSourceDirty,
  });

  console.log("");
  console.log("Docnav Public File Staging");
  console.log("Status: passed");
  console.log(`Public directory: ${result.publicDir}`);
  console.log(`Binary: ${result.publicBinaryPath}`);
  console.log(`Checksum: ${result.checksumPath}`);
  console.log("");
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
