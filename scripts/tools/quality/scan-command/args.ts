import { DEFAULT_CONFIG } from "../model/config.ts";
import { booleanOption, parseScriptArgs, stringOption, type ScriptArgToken } from "../../args.ts";
import { parsePositiveInteger } from "../../types.ts";
import type { QualityScanOptions } from "./types.ts";

export function parseArgs(argv = process.argv.slice(2)): QualityScanOptions {
  const parsed = parseScriptArgs({
    args: argv,
    options: {
      baseline: { type: "string" },
      "changed-files": { type: "string" },
      "top-n": { type: "string" },
      "artifact-dir": { type: "string" },
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
  return {
    artifactDir: stringOption(parsed.values, "artifact-dir") ?? DEFAULT_CONFIG.artifactDir,
    baseline,
    changedFiles: stringOption(parsed.values, "changed-files") ?? null,
    skipBaseline: resolveSkipBaseline(parsed.tokens, baseline === null),
    topN: parsePositiveInteger(stringOption(parsed.values, "top-n") ?? String(DEFAULT_CONFIG.report.topN), "--top-n")
  };
}

function resolveSkipBaseline(tokens: readonly ScriptArgToken[], defaultValue: boolean): boolean {
  let skipBaseline = defaultValue;
  for (const token of tokens) {
    if (token.kind !== "option") continue;
    if (token.name === "skip-baseline") {
      skipBaseline = true;
    } else if (token.name === "baseline" || token.name === "with-baseline") {
      skipBaseline = false;
    }
  }
  return skipBaseline;
}

function printHelp() {
  console.log(`
Docnav Code Quality Observability — 非阻断代码质量观测

Usage: node scripts/quality/scan.ts [options]

Options:
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

⚠️  Non-blocking: Lizard/scc/PMD CPD metric values do not cause command failure.
   Clippy remains the Rust blocking lint gate.
`);
}
