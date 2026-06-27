import fs from "node:fs";

import { errorMessage } from "./tools/errors.ts";
import { assert } from "./tools/validators/assertions.ts";
import { loadProtocolErrorDetailsRequirements } from "./tools/validators/protocol/error-detail-rules.ts";
import { toAbs } from "./tools/validators/repo/paths.ts";

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

  const protocolRequirements = loadProtocolErrorDetailsRequirements();
  assert(
    Object.keys(protocolRequirements).length > 0,
    "protocol response schema must declare projected error detail rules"
  );
  console.log("diagnostic projection checks ok");
}

try {
  main();
} catch (error: unknown) {
  console.error(errorMessage(error));
  process.exitCode = 1;
}
