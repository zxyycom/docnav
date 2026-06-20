import { describe, it } from "node:test";
import { strict as assert } from "node:assert";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { parseLizardCSV } from "./scanners/lizard.ts";
import {
  getPmdCpdLanguageForCodeArea,
  parsePmdCpdXml,
  parsePmdVersionOutput,
  scanWithPmdCpd
} from "./scanners/pmd-cpd/scanner.ts";
import {
  SCC_BY_FILE_CSV_HEADER,
  parseSccCSV
} from "./scanners/scc.ts";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");

// @case AUX-QUALITY-PARSER-001
describe("quality scanner output parsing", () => {
  it("parses scc 3.7 Provider paths and rejects unknown CSV headers", () => {
    const csv = [
      SCC_BY_FILE_CSV_HEADER,
      "Rust,crates/docnav/src/lib.rs,lib.rs,120,90,20,10,17,4096,70",
      "JavaScript,scripts/quality/scan.ts,scan.ts,60,50,5,5,8,2048,45"
    ].join("\n");

    const result = parseSccCSV(csv, "/repo");

    assert.equal(result.ok, true);
    assert.deepEqual(result.files!.map((f) => f.path), [
      "crates/docnav/src/lib.rs",
      "scripts/quality/scan.ts"
    ]);
    assert.equal(result.files![0]!.complexity.value, 17);
    assert.equal(
      parseSccCSV("Language,Location,Filename,Lines,Code,Comments,Blanks,Complexity,Bytes\n", "/repo").ok,
      false
    );
  });

  it("parses Lizard 1.23 function rows", () => {
    const csv = [
      "271,88,1887,7,326,\"generateWarnings@35-360@scripts/tools/quality/output/warnings/generator.ts\",\"scripts/tools/quality/output/warnings/generator.ts\",\"generateWarnings\",\"generateWarnings ( files , functions , duplicates , config , scope , baseline , comparisonStatus )\",35,360"
    ].join("\n");

    const result = parseLizardCSV(csv);

    assert.equal(result.ok, true);
    assert.deepEqual(result.functions![0], {
      name: "generateWarnings",
      file: "scripts/tools/quality/output/warnings/generator.ts",
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

  it("parses PMD CPD language, version, and XML output", () => {
    const output = [
      "PMD 7.25.0 (418f8b7413a1c0ecb3ee409b6edeedeae5c8df39, 2026-05-29T06:28:38Z)",
      "Java version: 21.0.11, vendor: Eclipse Adoptium"
    ].join("\n");
    const xml = [
      '<?xml version="1.0" encoding="UTF-8"?>',
      '<pmd-cpd>',
      '  <duplication lines="10" tokens="50">',
      '    <file path="/repo/crates/docnav/src/a.rs" line="10" endline="20"/>',
      '    <file path="/repo/crates/docnav/src/b.rs" line="5" endline="15"/>',
      "  </duplication>",
      "</pmd-cpd>"
    ].join("\n");

    const result = parsePmdCpdXml(xml, "/repo");

    assert.equal(getPmdCpdLanguageForCodeArea("node-production-scripts"), "typescript");
    assert.equal(parsePmdVersionOutput(output), "7.25.0");
    assert.equal(result.ok, true);
    assert.equal(result.fragments[0]!.tokenCount, 50);
    assert.deepEqual(result.fragments[0]!.locations.map((location) => location.path), [
      "crates/docnav/src/a.rs",
      "crates/docnav/src/b.rs"
    ]);
  });

  it("does not treat PMD CPD exit 4 without XML as a successful empty scan", () => {
    const toolConfig = createFakePmdToolConfig({ stdout: "", stderr: "", exitCode: 4 });

    try {
      const result = scanWithPmdCpd({
        files: ["scripts/a.ts", "scripts/b.ts"],
        cwd: REPO_ROOT,
        toolConfig,
        minimumTokens: 75,
        codeArea: "node-production-scripts",
        skipIfUnavailable: true
      });

      assert.equal(result.ok, false);
      if (!result.ok) {
        assert.equal(result.skipped, true);
        assert.match(result.error, /PMD CPD exit 4: no output/);
      }
    } finally {
      toolConfig.cleanup();
    }
  });
});

function createFakePmdToolConfig({
  stdout,
  stderr,
  exitCode
}: {
  exitCode: number;
  stderr: string;
  stdout: string;
}) {
  const tempDir = mkdtempSync(join(tmpdir(), "docnav-quality-pmd-"));
  const fakePmdPath = join(tempDir, "fake-pmd.ts");

  writeFileSync(fakePmdPath, `
process.stdout.write(${JSON.stringify(stdout)});
console.error(${JSON.stringify(stderr)});
process.exit(${JSON.stringify(exitCode)});
`, "utf8");

  return {
    command: process.execPath,
    args: [fakePmdPath],
    cleanup: () => rmSync(tempDir, { recursive: true, force: true })
  };
}
