import path from "node:path";

import { root, runScript } from "../tools/release-package/index.ts";

const withCargoBins = path.join(root, "scripts", "cargo", "with-bins.ts");

try {
  runScript(withCargoBins, [
    "--bin",
    "docnav-markdown:docnav-markdown:DOCNAV_MARKDOWN_BIN",
    "--",
    "bun",
    "test/docnav-markdown-smoke.ts",
  ]);
  runScript(withCargoBins, [
    "--bin",
    "docnav:docnav:DOCNAV_BIN",
    "--bin",
    "docnav-markdown:docnav-markdown:DOCNAV_MARKDOWN_BIN",
    "--",
    "bun",
    "test/docnav-core-smoke.ts",
  ]);
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
