import fs from "node:fs";
import path from "node:path";

import { publicBinaryName } from "./config.ts";
import { writeTextFile } from "../foundation/src/fs.ts";
import { copyExecutable, sha256File } from "./io.ts";
import { validateReleasePackage } from "./validation.ts";
import type { ManifestValidationOptions } from "./validation.ts";

type PublicStagingResult = {
  checksumPath: string;
  publicBinaryPath: string;
  publicDir: string;
};

export function stagePublicFiles(
  manifestPath: string,
  options: ManifestValidationOptions = {},
): PublicStagingResult {
  const validatedPackage = validateReleasePackage(manifestPath, options);
  const publicDir = path.join(
    path.dirname(validatedPackage.packageDir),
    "public",
  );

  fs.rmSync(publicDir, { recursive: true, force: true });
  try {
    const packageBinary = coreBinaryEntry(validatedPackage.fileEntries);
    const publicFileName = publicBinaryName(
      validatedPackage.manifest.version,
      validatedPackage.manifest.target,
    );
    const packageBinaryPath = path.join(
      validatedPackage.packageDir,
      packageBinary.path,
    );
    const publicBinaryPath = path.join(publicDir, publicFileName);

    copyExecutable(packageBinaryPath, publicBinaryPath);
    assertByteEquality(packageBinaryPath, publicBinaryPath);

    const publicHash = sha256File(publicBinaryPath);
    if (publicHash !== packageBinary.sha256) {
      throw new Error("public binary sha256 must match canonical package binary");
    }

    const checksumPath = `${publicBinaryPath}.sha256`;
    writeTextFile(
      checksumPath,
      `${publicHash}  ${publicFileName}\n`,
    );

    return {
      checksumPath,
      publicBinaryPath,
      publicDir,
    };
  } catch (error) {
    fs.rmSync(publicDir, { recursive: true, force: true });
    throw error;
  }
}

function coreBinaryEntry(
  fileEntries: ReturnType<typeof validateReleasePackage>["fileEntries"],
) {
  const entry = fileEntries.find(
    (candidate) => candidate.component === "core",
  );
  if (!entry || typeof entry.sha256 !== "string") {
    throw new Error("validated manifest missing core binary entry");
  }
  return entry;
}

function assertByteEquality(sourcePath: string, destinationPath: string): void {
  if (!fs.readFileSync(sourcePath).equals(fs.readFileSync(destinationPath))) {
    throw new Error("public binary bytes must match canonical package binary");
  }
}
