import path from "node:path";

import { root, runNodeScript } from "./tools/release-package/index.mjs";

const withCargoBins = path.join(root, "scripts", "with-cargo-bins.mjs");

try {
  runNodeScript(withCargoBins, [
    "--bin",
    "docnav-markdown",
    "docnav-markdown",
    "DOCNAV_MARKDOWN_BIN",
    "--",
    "node",
    "test/docnav-markdown-smoke.mjs",
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
    "test/docnav-core-smoke.mjs",
  ]);
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
