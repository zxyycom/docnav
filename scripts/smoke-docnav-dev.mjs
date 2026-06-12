import path from "node:path";

import { root, runNodeScript } from "./release-package.mjs";

const withCargoBins = path.join(root, "scripts", "with-cargo-bins.mjs");

try {
  runNodeScript(withCargoBins, [
    "--bin",
    "docnav-markdown",
    "docnav-markdown",
    "DOCNAV_MARKDOWN_BIN",
    "--",
    "node",
    "scripts/docnav-markdown-cli-smoke/index.mjs",
  ]);
  runNodeScript(withCargoBins, [
    "--bin",
    "docnav",
    "docnav",
    "DOCNAV_BIN",
    "--bin",
    "docnav-markdown",
    "docnav-markdown",
    "DOCNAV_MARKDOWN_BIN",
    "--",
    "node",
    "scripts/docnav-core-cli-smoke/index.mjs",
  ]);
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
