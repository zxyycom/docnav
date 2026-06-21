import { DEFAULT_CONFIG } from "../model/config.ts";
import { booleanOption, parsePositiveInteger, parseScriptArgs, stringOption, type ScriptArgToken } from "../../args.ts";
import {
  QUALITY_SCAN_PROFILES,
  type QualityScanOptions,
  type QualityScanProfile
} from "./command-model.ts";

const skipBaselineByToken: Partial<Record<string, boolean>> = {
  baseline: false,
  "skip-baseline": true,
  "with-baseline": false
};

export function parseArgs(argv = process.argv.slice(2)): QualityScanOptions {
  const parsed = parseScriptArgs({
    args: argv,
    options: {
      baseline: { type: "string" },
      "changed-files": { type: "string" },
      "top-n": { type: "string" },
      "artifact-dir": { type: "string" },
      profile: { type: "string" },
      "skip-baseline": { type: "boolean" },
      "with-baseline": { type: "boolean" },
      help: { type: "boolean" }
    }
  });

  if (booleanOption(parsed.values, "help")) {
    printHelp();
    process.exit(0);
  }

  const baseline = stringOption(parsed.values, "baseline") ?? null;
  const scanProfile = parseScanProfile(stringOption(parsed.values, "profile") ?? "full");
  if (scanProfile === "quick" && hasBaselineOption(parsed.tokens)) {
    throw new Error("quick quality check does not support baseline options; use --profile full for baseline comparison");
  }

  return {
    artifactDir: stringOption(parsed.values, "artifact-dir") ?? DEFAULT_CONFIG.artifactDir,
    baseline,
    changedFiles: stringOption(parsed.values, "changed-files") ?? null,
    scanProfile,
    skipBaseline: scanProfile === "quick" ? true : resolveSkipBaseline(parsed.tokens, baseline === null),
    topN: parsePositiveInteger(stringOption(parsed.values, "top-n") ?? String(DEFAULT_CONFIG.report.topN), "--top-n")
  };
}

function parseScanProfile(value: string): QualityScanProfile {
  if (QUALITY_SCAN_PROFILES.includes(value as QualityScanProfile)) {
    return value as QualityScanProfile;
  }
  throw new Error(`unknown quality scan profile: ${value}`);
}

function resolveSkipBaseline(tokens: readonly ScriptArgToken[], defaultValue: boolean): boolean {
  let skipBaseline = defaultValue;
  for (const token of tokens) {
    if (token.kind !== "option" || token.name === undefined) continue;
    skipBaseline = skipBaselineByToken[token.name] ?? skipBaseline;
  }
  return skipBaseline;
}

function hasBaselineOption(tokens: readonly ScriptArgToken[]): boolean {
  return tokens.some((token) =>
    token.kind === "option" &&
    (token.name === "baseline" || token.name === "with-baseline")
  );
}

function printHelp() {
  console.log(`
Docnav Code Quality Observability — 非阻断代码质量观测

Usage: node scripts/quality/scan.ts [options]

Options:
  --profile <quick|full>  Select quick or full quality check mode (default: full)
  --baseline <sha>        Generate baseline delta from an explicit commit SHA (opt-in)
  --with-baseline         Auto-detect and scan previous-code baseline (slower, opt-in)
  --changed-files <file>  File containing list of changed files (one per line)
  --top-n <n>             Top N for rankings (default: ${DEFAULT_CONFIG.report.topN})
  --artifact-dir <dir>    Artifact output directory (default: ${DEFAULT_CONFIG.artifactDir})
  --skip-baseline         Skip baseline commit detection and scan (default)
  --help                  Show this help

Output:
  metrics.json            Machine-readable quality metrics
  report.md               Human-readable Markdown summary
  warnings.ndjson         Changed warning records when baseline comparison is enabled (newline-delimited JSON)
  warnings-all.ndjson     Full warning records for local/governance use
  raw/                    Raw tool outputs (Lizard, scc, PMD CPD)

Profiles:
  quick                  Fast current-snapshot quality check; skips baseline and PMD CPD.
  full                   Full quality check; runs all configured scanners and optional baseline comparison.

Warning status:
  Warning records do not cause command failure, but the command prints
  "Quality check status: warning" with a short warning preview and report path.
  Clippy remains the Rust blocking lint gate.
`);
}
