import { describe, it } from "node:test";
import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { DEFAULT_CONFIG } from "../../scripts/quality/config.mjs";
import {
  SCC_BY_FILE_CSV_HEADER,
  buildSccArgs,
  scanWithScc,
  getSccVersion,
  parseSccCSV
} from "../../scripts/quality/tools/scc.mjs";
import {
  parseLizardCSV
} from "../../scripts/quality/tools/lizard.mjs";
import {
  buildPmdShellCommand,
  parseCpdXml,
  parsePmdVersionOutput
} from "../../scripts/quality/tools/cpd.mjs";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const QUALITY_SCAN_SCRIPT = join(REPO_ROOT, "scripts/quality/scan.mjs");

describe("CPD minimum tokens by code area", () => {
  it("rust-production has lowest minimum tokens", () => {
    const tokens = DEFAULT_CONFIG.pmdCpd.minimumTokens;
    const productionToken = tokens["rust-production"];
    const fixturesToken = tokens["fixtures-examples"];

    assert.ok(productionToken <= fixturesToken,
      `production (${productionToken}) should have <= tokens than fixtures (${fixturesToken})`);
  });

  it("generated has highest minimum tokens", () => {
    const tokens = DEFAULT_CONFIG.pmdCpd.minimumTokens;
    const generatedToken = tokens["generated"];
    const productionToken = tokens["rust-production"];

    assert.ok(generatedToken >= productionToken,
      `generated (${generatedToken}) should have >= tokens than production (${productionToken})`);
  });

  it("default minimum tokens is defined", () => {
    assert.ok(typeof DEFAULT_CONFIG.pmdCpd.defaultMinimumTokens === "number");
    assert.ok(DEFAULT_CONFIG.pmdCpd.defaultMinimumTokens > 0);
  });
});

// ═══════════════════════════════════════════════════════════════════════
// scc 3.7 CSV parsing 测试
// ═══════════════════════════════════════════════════════════════════════

describe("scc 3.7 CSV parsing", () => {
  it("exports the scc wrapper entry points", () => {
    assert.ok(typeof scanWithScc === "function");
    assert.ok(typeof getSccVersion === "function");
    assert.ok(typeof buildSccArgs === "function");
    assert.ok(typeof parseSccCSV === "function");
  });

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
      "JavaScript,scripts/quality/scan.mjs,scan.mjs,60,50,5,5,8,2048,45"
    ].join("\n");

    const result = parseSccCSV(csv, "/repo");

    assert.equal(result.ok, true);
    assert.deepEqual(result.files.map((f) => f.path), [
      "crates/docnav/src/lib.rs",
      "scripts/quality/scan.mjs"
    ]);
    assert.equal(result.files[0].complexity.value, 17);
    assert.deepEqual(result.aggregates.byLanguage.map((l) => [l.language, l.files, l.lines]), [
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
    assert.match(result.error, /expected scc 3\.7\.0 by-file CSV header/);
  });

  it("quality scan surfaces scc contract errors as fatal scan issues", () => {
    const result = runQualityScanWithFakeSccContractError();
    const output = `${result.stdout}\n${result.stderr}`;

    assert.equal(result.error, undefined, result.error?.message);
    assert.notEqual(result.status, 0, output);
    assert.match(output, /expected scc version 3\.7\.0, got "scc version 3\.6\.0"/);
    assert.match(output, /Fatal quality scan issues/);
  });
});

// ═══════════════════════════════════════════════════════════════════════
// Lizard 1.23 CSV parsing 测试
// ═══════════════════════════════════════════════════════════════════════

describe("Lizard CSV parsing", () => {
  it("parses Lizard 1.23 location, file, function, start, and end columns", () => {
    const csv = [
      "271,88,1887,7,326,\"generateWarnings@35-360@scripts/quality/warnings.mjs\",\"scripts/quality/warnings.mjs\",\"generateWarnings\",\"generateWarnings ( files , functions , duplicates , config , scope , baseline , comparisonStatus )\",35,360"
    ].join("\n");

    const result = parseLizardCSV(csv);

    assert.equal(result.ok, true);
    assert.equal(result.functions.length, 1);
    assert.deepEqual(result.functions[0], {
      name: "generateWarnings",
      file: "scripts/quality/warnings.mjs",
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
});

// ═══════════════════════════════════════════════════════════════════════
// PMD CPD XML 解析和 file-list 测试
// ═══════════════════════════════════════════════════════════════════════

describe("PMD CPD XML parsing and file-list", () => {
  it("scanWithCpd function accepts codeArea parameter", async () => {
    const mod = await import("../../scripts/quality/tools/cpd.mjs");
    assert.ok(typeof mod.scanWithCpd === "function");
    assert.ok(typeof mod.getCpdVersion === "function");
  });

  it("builds a cross-platform PMD shell command with quoted path arguments", () => {
    assert.equal(
      buildPmdShellCommand("pmd", [
        "cpd",
        "--file-list",
        "C:\\Temp\\docnav cpd files.txt",
        "--minimum-tokens",
        "75"
      ]),
      "pmd cpd --file-list \"C:\\Temp\\docnav cpd files.txt\" --minimum-tokens 75"
    );
  });

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

  it("CPD XML parser handles PMD file attributes in any order", () => {
    const xml = [
      '<?xml version="1.0" encoding="UTF-8"?>',
      '<pmd-cpd>',
      '  <duplication tokens="113" lines="14">',
      '    <file line="44" path="/repo/crates/docnav/src/a.rs" endline="57"/>',
      '    <file endline="92" path="/repo/crates/docnav/src/b.rs" line="79"/>',
      '  </duplication>',
      '</pmd-cpd>'
    ].join("\n");

    const result = parseCpdXml(xml, "/repo");

    assert.equal(result.ok, true);
    assert.equal(result.fragments.length, 1);
    assert.deepEqual(result.fragments[0].locations, [
      {
        path: "crates/docnav/src/a.rs",
        startLine: 44,
        endLine: 57,
        codeArea: "unknown"
      },
      {
        path: "crates/docnav/src/b.rs",
        startLine: 79,
        endLine: 92,
        codeArea: "unknown"
      }
    ]);
  });

  it("CPD XML parser rejects duplication entries without file locations", () => {
    const xml = [
      '<?xml version="1.0" encoding="UTF-8"?>',
      '<pmd-cpd>',
      '  <duplication tokens="113" lines="14">',
      '  </duplication>',
      '</pmd-cpd>'
    ].join("\n");

    const result = parseCpdXml(xml, "/repo");

    assert.equal(result.ok, false);
    assert.match(result.error, /must include at least one file location/);
  });

  it("CPD XML parser handles multiple duplications", () => {
    const xml = [
      '<?xml version="1.0" encoding="UTF-8"?>',
      '<pmd-cpd>',
      '  <duplication lines="15" tokens="80">',
      '    <file path="/repo/src/a.rs" line="1" endline="16"/>',
      '    <file path="/repo/src/b.rs" line="20" endline="35"/>',
      '  </duplication>',
      '  <duplication lines="8" tokens="30">',
      '    <file path="/repo/src/c.rs" line="5" endline="13"/>',
      '    <file path="/repo/src/d.rs" line="40" endline="48"/>',
      '    <file path="/repo/src/e.rs" line="60" endline="68"/>',
      '  </duplication>',
      '</pmd-cpd>'
    ].join("\n");

    const result = parseCpdXml(xml, "/repo");

    assert.equal(result.ok, true);
    assert.equal(result.fragments.length, 2);
    assert.equal(result.fragments[0].tokenCount, 80);
    assert.equal(result.fragments[1].tokenCount, 30);
    assert.equal(result.fragments[0].locations.length, 2);
    assert.equal(result.fragments[1].locations.length, 3);
  });

  it("CPD XML parser handles files without endline attribute", () => {
    const xml = [
      '<?xml version="1.0" encoding="UTF-8"?>',
      '<pmd-cpd>',
      '  <duplication lines="10" tokens="40">',
      '    <file path="/repo/src/a.rs" line="10"/>',
      '    <file path="/repo/src/b.rs" line="5"/>',
      '  </duplication>',
      '</pmd-cpd>'
    ].join("\n");

    const result = parseCpdXml(xml, "/repo");

    assert.equal(result.ok, true);
    assert.equal(result.fragments[0].locations[0].endLine, 20);
    assert.equal(result.fragments[0].locations[1].endLine, 15);
  });
});

function runQualityScanWithFakeSccContractError() {
  const tempDir = mkdtempSync(join(tmpdir(), "docnav-quality-scc-"));
  const fakeSccPath = join(tempDir, "fake-scc.mjs");
  const artifactDir = join(tempDir, "artifacts");

  writeFileSync(fakeSccPath, `
const args = process.argv.slice(2);

if (args.includes("--version")) {
  console.log("scc version 3.6.0");
  process.exit(0);
}

console.error("unexpected fake scc invocation: " + args.join(" "));
process.exit(1);
`, "utf8");

  try {
    return spawnSync(process.execPath, [
      QUALITY_SCAN_SCRIPT,
      "--skip-baseline",
      "--artifact-dir",
      artifactDir
    ], {
      cwd: REPO_ROOT,
      encoding: "utf8",
      env: {
        ...process.env,
        DOCNAV_LIZARD_CMD: "__docnav_missing_lizard__",
        DOCNAV_SCC_CMD: process.execPath,
        DOCNAV_SCC_ARGS: JSON.stringify([fakeSccPath])
      },
      maxBuffer: 1024 * 1024 * 16,
      windowsHide: true
    });
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
}
