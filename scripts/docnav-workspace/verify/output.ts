import { profiles } from "../checks/index.ts";
import type { Profile } from "../checks/index.ts";
import { formatCompletionLine, formatDurationMs } from "../results.ts";
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

export function printCompletionResult(
  result: Pick<CompletionResult, "check" | "durationMs" | "status" | "visibleOutput">,
  writeLine: (line: string) => void = console.log
): void {
  writeLine(formatCompletionLine(result));
  if (result.visibleOutput.length > 0) {
    writeLine(result.visibleOutput);
  }
}

export function printSummary({
  profile,
  totalChecks,
  completedResults,
  totalDurationMs,
  logPaths
}: SummaryInput): void {
  const failed = completedResults.filter((result) => result.status === "failed").length;
  const warnings = completedResults.filter((result) => result.status === "warning").length;
  const passed = completedResults.filter((result) => result.status === "passed").length;
  const status = failed > 0 ? "failed" : warnings > 0 ? "warning" : "passed";

  console.log("");
  console.log("Summary:");
  console.log(`  status: ${status}`);
  console.log(`  profile: ${profile}`);
  console.log(`  total checks: ${totalChecks}`);
  console.log(`  passed: ${passed}`);
  console.log(`  warning: ${warnings}`);
  console.log(`  failed: ${failed}`);
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
