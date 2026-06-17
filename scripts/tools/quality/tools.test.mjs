import { describe, it } from "node:test";
import { strict as assert } from "node:assert";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  parseLizardCSV
} from "./tools/lizard.mjs";
import {
  parseCpdXml,
  parsePmdVersionOutput
} from "./tools/cpd.mjs";
import {
  SCC_BY_FILE_CSV_HEADER,
  buildSccArgs,
  getSccVersion,
  parseSccCSV
} from "./tools/scc.mjs";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");

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
      "JavaScript,scripts/quality-scan.mjs,scan.mjs,60,50,5,5,8,2048,45"
    ].join("\n");

    const result = parseSccCSV(csv, "/repo");

    assert.equal(result.ok, true);
    assert.deepEqual(result.files.map((f) => f.path), [
      "crates/docnav/src/lib.rs",
      "scripts/quality-scan.mjs"
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
      "271,88,1887,7,326,\"generateWarnings@35-360@scripts/tools/quality/warnings.mjs\",\"scripts/tools/quality/warnings.mjs\",\"generateWarnings\",\"generateWarnings ( files , functions , duplicates , config , scope , baseline , comparisonStatus )\",35,360"
    ].join("\n");

    const result = parseLizardCSV(csv);

    assert.equal(result.ok, true);
    assert.equal(result.functions.length, 1);
    assert.deepEqual(result.functions[0], {
      name: "generateWarnings",
      file: "scripts/tools/quality/warnings.mjs",
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
      "10,3,100,1,12,22,scripts/tools/quality/warnings.mjs,generateWarnings"
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

function createFakeSccToolConfig(versionOutput) {
  const tempDir = mkdtempSync(join(tmpdir(), "docnav-quality-scc-"));
  const fakeSccPath = join(tempDir, "fake-scc.mjs");

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
