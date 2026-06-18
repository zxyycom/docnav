import { describe, it } from "node:test";
import { strict as assert } from "node:assert";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  parseLizardCSV
} from "./tools/lizard.ts";
import {
  parseCpdXml,
  parsePmdVersionOutput
} from "./tools/cpd.ts";
import {
  SCC_BY_FILE_CSV_HEADER,
  buildSccArgs,
  getSccVersion,
  parseSccCSV
} from "./tools/scc.ts";
import { buildFingerprints } from "./files.ts";
import { DEFAULT_CONFIG } from "./config.ts";
import { createEmptyMetrics } from "./schema.ts";
import type { FileMetric, QualityMetrics, WarningRecord } from "./schema.ts";
import { changedFilesSection } from "./report/findings.ts";
import { comparisonInfo, scanInfo } from "./report/summary.ts";
import { generateWarningChannels } from "./warnings.ts";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");

describe("quality fingerprints", () => {
  it("uses the same code area fingerprint for LF and CRLF text", () => {
    const tempDir = mkdtempSync(join(tmpdir(), "docnav-quality-fingerprint-"));
    const fileMap = new Map([["typescript", ["src/example.ts"]]]);

    try {
      writeFixtureFile(tempDir, "lf/src/example.ts", "export const value = 1;\nconsole.log(value);\n");
      writeFixtureFile(tempDir, "crlf/src/example.ts", "export const value = 1;\r\nconsole.log(value);\r\n");

      const lfFingerprint = buildFingerprints(fileMap, join(tempDir, "lf")).typescript;
      const crlfFingerprint = buildFingerprints(fileMap, join(tempDir, "crlf")).typescript;

      assert.deepEqual(crlfFingerprint, lfFingerprint);
    } finally {
      rmSync(tempDir, { recursive: true, force: true });
    }
  });
});

describe("scc 3.7 CSV parsing", () => {
  it("uses the scc 3.7 by-file CSV invocation shape", () => {
    const args = buildSccArgs({
      includePaths: ["crates", "scripts", "test"],
      excludeDirs: ["target", "node_modules"],
      toolArgs: []
    });

    assert.deepEqual(args.slice(0, 3), ["--by-file", "--format", "csv"]);
    assert.ok(args.includes("--exclude-dir"));
    assert.deepEqual(args.slice(-3), ["crates", "scripts", "test"]);
  });

  it("requires the pinned scc 3.7 by-file CSV header", () => {
    assert.equal(
      SCC_BY_FILE_CSV_HEADER,
      "Language,Provider,Filename,Lines,Code,Comments,Blanks,Complexity,Bytes,ULOC"
    );
  });

  it("parses scc 3.7 Provider paths directly", () => {
    const csv = [
      SCC_BY_FILE_CSV_HEADER,
      "Rust,crates/docnav/src/lib.rs,lib.rs,120,90,20,10,17,4096,70",
      "JavaScript,scripts/quality-scan.ts,scan.ts,60,50,5,5,8,2048,45"
    ].join("\n");

    const result = parseSccCSV(csv, "/repo");

    assert.equal(result.ok, true);
    assert.deepEqual(result.files!.map((f) => f.path), [
      "crates/docnav/src/lib.rs",
      "scripts/quality-scan.ts"
    ]);
    assert.equal(result.files![0]!.complexity.value, 17);
    assert.deepEqual(result.aggregates!.byLanguage.map((l) => [l.language, l.files, l.lines]), [
      ["Rust", 1, 120],
      ["JavaScript", 1, 60]
    ]);
  });

  it("rejects the old scc Location header", () => {
    const csv = [
      "Language,Location,Filename,Lines,Code,Comments,Blanks,Complexity,Bytes",
      "Rust,crates/docnav/src,lib.rs,120,90,20,10,17,4096"
    ].join("\n");

    const result = parseSccCSV(csv, "/repo");

    assert.equal(result.ok, false);
    assert.match(result.error!, /expected scc 3\.7\.0 by-file CSV header/);
  });

  it("reports scc contract errors during tool validation", () => {
    const toolConfig = createFakeSccToolConfig("scc version 3.6.0");
    try {
      const version = getSccVersion({
        cwd: REPO_ROOT,
        toolConfig
      });

      assert.deepEqual(version, {
        ok: false,
        error: 'expected scc version 3.7.0, got "scc version 3.6.0"',
        reason: "contract-error"
      });

    } finally {
      toolConfig.cleanup();
    }
  });
});

describe("Lizard CSV parsing", () => {
  it("parses Lizard 1.23 location, file, function, start, and end columns", () => {
    const csv = [
      "271,88,1887,7,326,\"generateWarnings@35-360@scripts/tools/quality/warnings.ts\",\"scripts/tools/quality/warnings.ts\",\"generateWarnings\",\"generateWarnings ( files , functions , duplicates , config , scope , baseline , comparisonStatus )\",35,360"
    ].join("\n");

    const result = parseLizardCSV(csv);

    assert.equal(result.ok, true);
    assert.equal(result.functions!.length, 1);
    assert.deepEqual(result.functions![0], {
      name: "generateWarnings",
      file: "scripts/tools/quality/warnings.ts",
      codeArea: "unknown",
      startLine: 35,
      endLine: 360,
      lines: 271,
      parameterCount: 7,
      cyclomaticComplexity: {
        value: 88,
        source: "lizard"
      },
      isChanged: false
    });
  });

  it("ignores rows that are not the Lizard 1.23 CSV shape", () => {
    const csv = [
      "10,3,100,1,12,22,scripts/tools/quality/warnings.ts,generateWarnings"
    ].join("\n");

    const result = parseLizardCSV(csv);

    assert.equal(result.ok, true);
    assert.deepEqual(result.functions, []);
  });
});

describe("PMD CPD XML parsing", () => {
  it("parses the PMD version line without banner art or Java runtime details", () => {
    const output = [
      "  ████                            ████",
      "  ██                                ██",
      "PMD 7.25.0 (418f8b7413a1c0ecb3ee409b6edeedeae5c8df39, 2026-05-29T06:28:38Z)",
      "Java version: 21.0.11, vendor: Eclipse Adoptium"
    ].join("\n");

    assert.equal(parsePmdVersionOutput(output), "7.25.0");
  });

  it("CPD XML parser handles empty xml gracefully", () => {
    assert.deepEqual(parseCpdXml("", "/repo"), { ok: true, fragments: [] });
    assert.deepEqual(parseCpdXml("<pmd-cpd></pmd-cpd>", "/repo"), { ok: true, fragments: [] });
  });

  it("CPD XML parser extracts duplication fragments", () => {
    const xml = [
      '<?xml version="1.0" encoding="UTF-8"?>',
      '<pmd-cpd>',
      '  <duplication lines="10" tokens="50">',
      '    <file path="/repo/crates/docnav/src/a.rs" line="10" endline="20"/>',
      '    <file path="/repo/crates/docnav/src/b.rs" line="5" endline="15"/>',
      '  </duplication>',
      '</pmd-cpd>'
    ].join("\n");

    const result = parseCpdXml(xml, "/repo");

    assert.equal(result.ok, true);
    assert.equal(result.fragments.length, 1);
    assert.equal(result.fragments[0].lineCount, 10);
    assert.equal(result.fragments[0].tokenCount, 50);
    assert.deepEqual(result.fragments[0].locations, [
      {
        path: "crates/docnav/src/a.rs",
        startLine: 10,
        endLine: 20,
        codeArea: "unknown"
      },
      {
        path: "crates/docnav/src/b.rs",
        startLine: 5,
        endLine: 15,
        codeArea: "unknown"
      }
    ]);
  });
});

describe("quality warning channels", () => {
  it("keeps full quality debt separate from changed and regression warnings", () => {
    const currentFiles: FileMetric[] = [
      qualityFile("src/changed.ts", { isChanged: true, lines: 480, complexity: 45 }),
      qualityFile("src/legacy.ts", { isChanged: false, lines: 700, complexity: 60 })
    ];
    const baselineFiles: FileMetric[] = [
      qualityFile("src/changed.ts", { isChanged: false, lines: 300, complexity: 25 }),
      qualityFile("src/legacy.ts", { isChanged: false, lines: 700, complexity: 60 })
    ];

    const channels = generateWarningChannels({
      files: currentFiles,
      functions: [],
      duplicates: [],
      config: DEFAULT_CONFIG,
      scope: { changed: true, changedFiles: ["src/changed.ts"] },
      baseline: { files: baselineFiles, functions: [], duplicates: [] },
      comparisonStatus: "compared"
    });

    assert.equal(channels.all.length, 4);
    assert.deepEqual(channels.changed.map((warning) => warning.path), [
      "src/changed.ts",
      "src/changed.ts"
    ]);
    assert.deepEqual(channels.regressions.map((warning) => warning.ruleId), [
      "scc-file-lines",
      "scc-file-complexity"
    ]);
  });

  it("still reports all warnings when changed annotations are suppressed", () => {
    const channels = generateWarningChannels({
      files: [qualityFile("src/legacy.ts", { isChanged: false, lines: 700, complexity: 60 })],
      functions: [],
      duplicates: [],
      config: DEFAULT_CONFIG,
      scope: { changed: false, changedFiles: [] },
      baseline: null,
      comparisonStatus: "input-unchanged"
    });

    assert.equal(channels.all.length, 2);
    assert.deepEqual(channels.changed, []);
    assert.deepEqual(channels.regressions, []);
  });
});

describe("quality report changed file watchlist", () => {
  it("shows only risk-ranked changed files and keeps the full count in summary text", () => {
    const metrics = qualityMetrics();
    metrics.fileMetrics = [
      qualityFile("src/risky.ts", { isChanged: true, lines: 480, complexity: 45 }),
      qualityFile("src/duplicate.ts", { isChanged: true, lines: 120, complexity: 5 }),
      qualityFile("src/quiet.ts", { isChanged: true, lines: 80, complexity: 2 })
    ];
    metrics.duplicateCode = [{
      id: 1,
      tokenCount: 90,
      lineCount: 12,
      codeAreas: ["node-production-scripts"],
      hitsChangedScope: true,
      locations: [{
        path: "src/duplicate.ts",
        startLine: 10,
        endLine: 22,
        codeArea: "node-production-scripts"
      }]
    }];
    metrics.warnings = {
      all: [warning("src/risky.ts", "scc-file-lines", 480, 180)],
      changed: [warning("src/risky.ts", "scc-file-lines", 480, 180)],
      regressions: [warning("src/risky.ts", "scc-file-lines", 480, 180)]
    };

    const section = changedFilesSection(metrics, 10);

    assert.match(section, /Changed files: 3 total, 2 shown by risk ranking/);
    assert.match(section, /src\/risky\.ts/);
    assert.match(section, /src\/duplicate\.ts/);
    assert.doesNotMatch(section, /src\/quiet\.ts/);
  });
});

describe("quality report commit display", () => {
  it("shows current commit hash with its title", () => {
    const metrics = qualityMetrics();
    metrics.metadata.commitSha = "abc123";
    metrics.metadata.commitTitle = "add quality warning channels";

    const section = scanInfo(metrics, { timeZone: "UTC" });

    assert.match(section, /- \*\*Commit\*\*: `abc123` - add quality warning channels/);
  });

  it("shows baseline commit hash with its title", () => {
    const metrics = qualityMetrics();
    metrics.comparisonStatus = "compared";
    metrics.baseline = {
      status: "generated",
      commitSha: "def456",
      commitDate: "2026-06-18T01:02:03Z",
      metadata: {
        commitSha: "def456",
        commitDate: "2026-06-18T01:02:03Z",
        commitTitle: "implement quality observability",
        selectionReason: "nearest-code-commit",
        configVersion: DEFAULT_CONFIG.version,
        toolMetadata: []
      }
    };

    const section = comparisonInfo(metrics);

    assert.match(section, /- \*\*Baseline commit\*\*: `def456` - implement quality observability/);
  });
});

function createFakeSccToolConfig(versionOutput: string) {
  const tempDir = mkdtempSync(join(tmpdir(), "docnav-quality-scc-"));
  const fakeSccPath = join(tempDir, "fake-scc.ts");

  writeFileSync(fakeSccPath, `
const args = process.argv.slice(2);

if (args.includes("--version")) {
  console.log(${JSON.stringify(versionOutput)});
  process.exit(0);
}

console.error("unexpected fake scc invocation: " + args.join(" "));
process.exit(1);
`, "utf8");

  return {
    command: process.execPath,
    args: [fakeSccPath],
    cleanup: () => rmSync(tempDir, { recursive: true, force: true })
  };
}

function writeFixtureFile(rootDir: string, relPath: string, content: string): void {
  const absPath = join(rootDir, relPath);
  mkdirSync(dirname(absPath), { recursive: true });
  writeFileSync(absPath, content, "utf8");
}

function qualityMetrics(): QualityMetrics {
  return createEmptyMetrics({
    repository: REPO_ROOT,
    commitSha: "test",
    configVersion: DEFAULT_CONFIG.version,
    tools: [],
    scope: {
      include: [],
      excludeDirs: [],
      generatedFiles: []
    }
  });
}

function qualityFile(
  path: string,
  options: { complexity: number; isChanged: boolean; lines: number }
): FileMetric {
  return {
    path,
    language: "TypeScript",
    codeArea: "node-production-scripts",
    lines: options.lines,
    codeLines: options.lines,
    complexity: { value: options.complexity, source: "scc" },
    isChanged: options.isChanged
  };
}

function warning(
  path: string,
  ruleId: string,
  value: number,
  deltaValue: number
): WarningRecord {
  return {
    level: "warning",
    ruleId,
    sourceTool: "scc",
    path,
    line: null,
    codeArea: "node-production-scripts",
    metric: "lines",
    value,
    comparisonBasis: "delta",
    baselineValue: value - deltaValue,
    deltaValue,
    isChanged: true,
    message: "test warning"
  };
}
