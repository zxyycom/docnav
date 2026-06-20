import { checks } from "./definitions.ts";
import { PROFILE_FULL, PROFILE_REQUIRED, profiles } from "./model.ts";
import type { CheckTask, Profile } from "./model.ts";

export { asCheckTask } from "./normalization.ts";
export { checks } from "./definitions.ts";
export { PROFILE_FULL, PROFILE_REQUIRED, profiles } from "./model.ts";
export type { CheckDefinition, CheckReportRef, CheckTask, Profile } from "./model.ts";

export function checksForProfile(profile: Profile): CheckTask[] {
  assertProfile(profile);
  if (profile === PROFILE_REQUIRED) {
    return checks.filter((check) => check.type === PROFILE_REQUIRED);
  }
  return checks.filter((check) => check.type === PROFILE_REQUIRED || check.type === PROFILE_FULL);
}

export function reportCountForChecks(checkList: readonly CheckTask[]): number {
  return new Set(checkList.map(reportIdForCheck)).size;
}

export function reportIdForCheck(check: CheckTask): string {
  return check.reportId ?? check.id;
}

export function reportLabelForCheck(check: CheckTask): string {
  return check.reportLabel ?? check.label;
}

export function parseProfile(profile: string): Profile {
  assertProfile(profile);
  return profile;
}

export function assertProfile(profile: string): asserts profile is Profile {
  if (!Object.hasOwn(profiles, profile)) {
    throw new Error(`unknown verification profile: ${profile}`);
  }
}

export function visibleOutputLines(check: CheckTask, output: string): string[] {
  return lines(output).filter((line) => !isIgnoredOutput(check, line));
}

export function isIgnoredOutput(check: Pick<CheckTask, "ignoreOutput">, line: string): boolean {
  return (check.ignoreOutput ?? []).some((pattern) => pattern.test(line));
}

function lines(output: string): string[] {
  return output.split(/\r?\n/).filter((line) => line.length > 0);
}
