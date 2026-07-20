import {
  parseScriptArgs,
  stringOption,
} from "../tools/foundation/src/args.ts";
import {
  resolveCandidateExpectations,
  validateReleaseCandidate,
} from "../tools/release-package/candidate.ts";

try {
  const parsed = parseScriptArgs({
    args: process.argv.slice(2),
    options: {
      "candidate-root": { type: "string" },
      tag: { type: "string" },
    },
  });
  const candidateRoot = stringOption(parsed.values, "candidate-root");
  if (!candidateRoot) {
    throw new Error(
      "candidate validation requires --candidate-root <version-root>",
    );
  }
  const tagName = stringOption(parsed.values, "tag") ?? null;
  const result = validateReleaseCandidate(
    candidateRoot,
    resolveCandidateExpectations(tagName),
  );

  console.log("");
  console.log("Docnav Release Candidate Validation");
  console.log("Status: passed");
  console.log(`Version: ${result.version}`);
  console.log(`Commit: ${result.gitCommit}`);
  console.log(`Candidate: ${result.candidateRoot}`);
  for (const target of result.targets) {
    console.log(`Target: ${target.target}`);
    console.log(`  Manifest SHA-256: ${target.manifestHash}`);
    console.log(`  Package binary SHA-256: ${target.packageBinaryHash}`);
    console.log(`  Public binary SHA-256: ${target.publicBinaryHash}`);
  }
  console.log("");
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
