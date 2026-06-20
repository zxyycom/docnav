import { describe, it } from "node:test";
import { strict as assert } from "node:assert";

import { createEmptyMetrics, type QualityMetrics, type WarningRecord } from "../schema.ts";
import { changedFilesSection } from "./findings.ts";
import { fileRankings, functionSizeRankings } from "./rankings.ts";

// @case AUX-QUALITY-REPORT-001
describe("quality report", () => {
  it("keeps changed-file watchlist useful without baseline annotations", () => {
    const metrics = qualityMetrics();
    metrics.comparisonStatus = "baseline-unavailable";
    metrics.fileMetrics = [
      qualityFile("src/risky.ts", { isChanged: true, lines: 480, complexity: 45 }),
      qualityFile("src/quiet.ts", { isChanged: true, lines: 80, complexity: 2 })
    ];
    metrics.warnings = {
      all: [warning("src/risky.ts", "scc-file-code-lines", 480)],
      changed: [],
      regressions: []
    };

    const section = changedFilesSection(metrics, 10);

    assert.match(section, /Changed files: 2 total, 1 shown by risk ranking/);
    assert.match(section, /src\/risky\.ts/);
    assert.doesNotMatch(section, /src\/quiet\.ts/);
  });

  it("sorts rankings by metric without mutating scanner output order", () => {
    const metrics = qualityMetrics();
    metrics.fileMetrics = [
      qualityFile("src/a-small.ts", { isChanged: false, lines: 10, complexity: 1 }),
      qualityFile("src/b-large.ts", { isChanged: false, lines: 500, complexity: 3 }),
      qualityFile("src/c-medium.ts", { isChanged: false, lines: 200, complexity: 2 })
    ];
    metrics.functionMetrics = [
      qualityFunction("small", "src/a-small.ts", { lines: 5, complexity: 1 }),
      qualityFunction("large", "src/b-large.ts", { lines: 80, complexity: 3 }),
      qualityFunction("medium", "src/c-medium.ts", { lines: 40, complexity: 2 })
    ];
    const originalFileOrder = metrics.fileMetrics.map((file) => file.path);
    const originalFunctionOrder = metrics.functionMetrics.map((func) => func.name);

    const files = fileRankings(metrics, 2);
    const functions = functionSizeRankings(metrics, 2);

    assert.ok(files.indexOf("src/b-large.ts") < files.indexOf("src/c-medium.ts"));
    assert.doesNotMatch(files, /src\/a-small\.ts/);
    assert.ok(functions.indexOf("large") < functions.indexOf("medium"));
    assert.doesNotMatch(functions, /small/);
    assert.deepEqual(metrics.fileMetrics.map((file) => file.path), originalFileOrder);
    assert.deepEqual(metrics.functionMetrics.map((func) => func.name), originalFunctionOrder);
  });
});

function qualityMetrics(): QualityMetrics {
  return createEmptyMetrics({
    repository: "/repo",
    commitSha: "test",
    configVersion: "quality-observability-v1",
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
): QualityMetrics["fileMetrics"][number] {
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

function qualityFunction(
  name: string,
  file: string,
  options: { complexity: number; lines: number }
): QualityMetrics["functionMetrics"][number] {
  return {
    name,
    file,
    codeArea: "node-production-scripts",
    startLine: 1,
    endLine: options.lines,
    lines: options.lines,
    parameterCount: 1,
    cyclomaticComplexity: { value: options.complexity, source: "lizard" },
    isChanged: false
  };
}

function warning(path: string, ruleId: string, value: number): WarningRecord {
  return {
    level: "warning",
    ruleId,
    sourceTool: "scc",
    path,
    line: null,
    codeArea: "node-production-scripts",
    metric: "code-lines",
    value,
    comparisonBasis: "changed-scope",
    baselineValue: null,
    deltaValue: null,
    isChanged: true,
    message: "test warning"
  };
}
