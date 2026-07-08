import { booleanOption, parsePositiveInteger, parseScriptArgs, stringOption } from "../../tools/foundation/src/args.ts";
import { PROFILE_FULL, parseProfile } from "../checks/index.ts";
import type { Profile } from "../checks/index.ts";

export interface VerificationOptions {
  help: boolean;
  profile: Profile;
  concurrency: number | undefined;
}

export function parseArgs(argv: string[]): VerificationOptions {
  const parsed = parseScriptArgs({
    args: argv,
    options: {
      concurrency: { type: "string" },
      help: { type: "boolean", short: "h" },
      profile: { type: "string" }
    }
  });

  return {
    help: booleanOption(parsed.values, "help"),
    profile: parseProfile(stringOption(parsed.values, "profile") ?? PROFILE_FULL),
    concurrency: resolveVerificationConcurrency(stringOption(parsed.values, "concurrency"))
  };
}

export function resolveVerificationConcurrency(value = process.env.DOCNAV_VERIFY_CONCURRENCY) {
  if (value === undefined || value === null || value === "") {
    return undefined;
  }
  return parsePositiveInteger(value, "verification concurrency");
}
