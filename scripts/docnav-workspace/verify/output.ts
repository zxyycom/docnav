import { profiles } from "../checks/index.ts";
import type { Profile } from "../checks/index.ts";
import { formatDurationMs } from "../results.ts";
import type { CompletionResult } from "../results.ts";
import { relativeLogPath } from "./paths.ts";

interface SummaryInput {
  profile: Profile;
  totalChecks: number;
  completedResults: readonly CompletionResult[];
  totalDurationMs: number;
  logPaths: readonly string[];
}

export function printHeader(profile: Profile, totalChecks: number): void {
  console.log("");
  console.log("Docnav Workspace Verification");
  console.log(`Profile: ${profile}`);
  console.log(`Total checks: ${totalChecks}`);
  console.log("");
  console.log("Checks:");
}

export function printSummary({
  profile,
  totalChecks,
  completedResults,
  totalDurationMs,
  logPaths
}: SummaryInput): void {
  console.log("");
  console.log("Summary:");
  console.log(`  status: ${completedResults.some((result) => !result.ok) ? "failed" : "passed"}`);
  console.log(`  profile: ${profile}`);
  console.log(`  total checks: ${totalChecks}`);
  console.log(`  passed: ${completedResults.filter((result) => result.ok).length}`);
  console.log(`  failed: ${completedResults.filter((result) => !result.ok).length}`);
  console.log(`  duration: ${formatDurationMs(totalDurationMs)}`);
  console.log(`  log: ${relativeLogPath(logPaths[0])}`);
  console.log("");
}

export function printUsage(writeLine: (line: string) => void): void {
  writeLine("Usage: node scripts/docnav-workspace/verify.ts [--profile required|full] [--concurrency <n>]");
  writeLine("");
  writeLine("Profiles:");
  for (const [name, profile] of Object.entries(profiles)) {
    writeLine(`  - ${name}: ${profile.description}`);
  }
}
