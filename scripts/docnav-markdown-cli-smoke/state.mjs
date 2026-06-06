import fs from "node:fs";
import path from "node:path";

import { root } from "./config.mjs";

export const smokeState = {
  commandRecords: [],
  testResults: [],
  startedAt: new Date(),
  binaryPath: resolveBinaryPath(argValue("--bin") ?? process.env.DOCNAV_MARKDOWN_BIN),
  validators: null,
  normalRef: null,
  normalReadableReadResult: null,
  normalReadableFindResult: null,
  normalProtocolReadResult: null
};

function argValue(flag) {
  const index = process.argv.indexOf(flag);
  if (index === -1) {
    return null;
  }
  return process.argv[index + 1] ?? null;
}

function resolveBinaryPath(value) {
  if (!value) {
    return null;
  }
  const resolved = path.resolve(root, value);
  if (fs.existsSync(resolved)) {
    return resolved;
  }
  if (process.platform === "win32" && !resolved.toLowerCase().endsWith(".exe")) {
    const exePath = `${resolved}.exe`;
    if (fs.existsSync(exePath)) {
      return exePath;
    }
  }
  return resolved;
}

