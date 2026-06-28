import fs from "node:fs";
import { spawnSync } from "node:child_process";

import { errorMessage } from "./tools/errors.ts";
import { assert } from "./tools/validators/assertions.ts";
import { root, toAbs } from "./tools/validators/repo/paths.ts";

const removedErrorRuleSources = [
  "docs/protocol/error-rules.json",
  "crates/docnav-protocol/src/generated.rs",
  "crates/docnav-protocol/src/generated/error_rules.rs",
  "scripts/generate-error-rules.ts",
  "scripts/tools/validators/generated/error/rules.ts"
];

function main(): void {
  for (const relPath of removedErrorRuleSources) {
    assert(!fs.existsSync(toAbs(relPath)), `${relPath} must not exist`);
  }

  const protocolRequirements = spawnSync(
    "cargo",
    [
      "test",
      "-p",
      "docnav-protocol",
      "protocol_response_schema_error_projection_matches_diagnostic_rules"
    ],
    { cwd: root, encoding: "utf8" }
  );
  assert(
    protocolRequirements.status === 0,
    `diagnostic owner/schema projection test failed\n${protocolRequirements.stdout}${protocolRequirements.stderr}`
  );
  console.log("diagnostic projection checks ok");
}

try {
  main();
} catch (error: unknown) {
  console.error(errorMessage(error));
  process.exitCode = 1;
}
