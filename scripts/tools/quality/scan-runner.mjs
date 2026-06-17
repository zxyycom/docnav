/**
 * Current revision quality scan runner.
 */

import { writeFileSync } from "node:fs";
import { join } from "node:path";

import { scanWithLizard } from "./tools/lizard.mjs";
import { scanWithScc } from "./tools/scc.mjs";
import { scanWithCpd } from "./tools/cpd.mjs";
import { classifyFile, isExcluded } from "./classify.mjs";
import { buildAggregates } from "./aggregate.mjs";

export function scanCurrentRevision({
  metrics,
  toolResults,
  scanFiles,
  changedFiles,
  fileMap,
  rawDir,
  fatalIssues,
  root,
  config
}) {
  runSccScan({ metrics, toolResults, scanFiles, changedFiles, rawDir, fatalIssues, root, config });
  runLizardScan({ metrics, toolResults, scanFiles, changedFiles, rawDir, fatalIssues, root, config });
  runCpdScan({ metrics, toolResults, fileMap, changedFiles, rawDir, root, config });

  metrics.aggregates = buildAggregates({
    fileMetrics: metrics.fileMetrics,
    functionMetrics: metrics.functionMetrics,
    duplicateCode: metrics.duplicateCode,
    byLanguage: metrics.aggregates.byLanguage,
    config
  });
}

function runSccScan({ metrics, toolResults, scanFiles, changedFiles, rawDir, fatalIssues, root, config }) {
  if (!isToolAvailable(toolResults, "scc")) return;

  console.log("Running scc...");
  const sccResult = scanWithScc({
    cwd: root,
    includePaths: scanFiles,
    excludeDirs: config.excludeDirs,
    toolConfig: config.tools.scc
  });

  if (sccResult.ok) {
    for (const file of sccResult.files) {
      file.codeArea = classifyFile(file.path, config.codeAreas, config.generatedFiles);
      file.isChanged = isInChangedScope(file.path, changedFiles);
    }

    metrics.fileMetrics = sccResult.files.filter(
      (f) => !isExcluded(f.path, config.excludeDirs, config.generatedFiles)
    );
    metrics.aggregates.byLanguage = sccResult.aggregates.byLanguage;
    metrics.aggregates.byCodeArea = buildFileAreaAggregates(metrics.fileMetrics, config);
    metrics.aggregates.overall = {
      totalFiles: metrics.fileMetrics.length,
      totalLines: metrics.fileMetrics.reduce((s, f) => s + f.lines, 0),
      totalCodeLines: metrics.fileMetrics.reduce((s, f) => s + (f.codeLines || 0), 0),
      totalFunctions: 0
    };

    console.log(`  scc: ${metrics.fileMetrics.length} files, ${metrics.aggregates.byLanguage.length} languages`);
  } else {
    fatalIssues.push({ tool: "scc", phase: "current-scan", error: sccResult.error });
    console.log(`  ❌ scc execution/config/schema error: ${sccResult.error}`);
  }

  writeFileSync(join(rawDir, "scc-output.json"), JSON.stringify(metrics.fileMetrics, null, 2), "utf8");
}

function runLizardScan({ metrics, toolResults, scanFiles, changedFiles, rawDir, fatalIssues, root, config }) {
  if (!isToolAvailable(toolResults, "lizard")) return;

  console.log("Running Lizard...");

  const targetFiles = scanFiles.filter(
    (f) => (f.endsWith(".rs") || f.endsWith(".mjs") || f.endsWith(".js")) &&
      !isExcluded(f, config.excludeDirs, config.generatedFiles)
  );
  console.log(`  Lizard targets: ${targetFiles.length} files`);

  const { functions: allFunctions, errors } = scanLizardBatches({ targetFiles, root, config });
  for (const error of errors) {
    fatalIssues.push({ tool: "lizard", phase: "current-scan", error });
    console.log(`  ❌ Lizard execution/config/schema error: ${error}`);
  }

  for (const func of allFunctions) {
    func.codeArea = classifyFile(func.file, config.codeAreas, config.generatedFiles);
    func.isChanged = isInChangedScope(func.file, changedFiles);
  }

  metrics.functionMetrics = allFunctions.filter(
    (f) => !isExcluded(f.file, config.excludeDirs, config.generatedFiles)
  );
  updateFunctionCounts(metrics);

  console.log(`  Lizard: ${metrics.functionMetrics.length} functions`);

  writeFileSync(
    join(rawDir, "lizard-functions.json"),
    JSON.stringify(metrics.functionMetrics, null, 2),
    "utf8"
  );
}

function runCpdScan({ metrics, toolResults, fileMap, changedFiles, rawDir, root, config }) {
  if (!isToolAvailable(toolResults, "pmd-cpd")) {
    console.log("  CPD not available, skipping duplicate detection");
    return;
  }

  console.log("Running PMD CPD...");

  const allFragments = [];

  for (const [area, areaFiles] of fileMap.entries()) {
    const targetFiles = areaFiles.filter(
      (f) => !isExcluded(f, config.excludeDirs, config.generatedFiles)
    );

    if (targetFiles.length < 2) {
      console.log(`  CPD ${area}: too few files (${targetFiles.length}), skipping`);
      continue;
    }

    const minTokens = config.pmdCpd.minimumTokens[area] ?? config.pmdCpd.defaultMinimumTokens;
    console.log(`  CPD ${area}: ${targetFiles.length} files, minimum tokens=${minTokens}`);

    const cpdResult = scanWithCpd({
      files: targetFiles,
      cwd: root,
      toolConfig: config.tools.pmdCpd,
      minimumTokens: minTokens,
      codeArea: area,
      skipIfUnavailable: true
    });

    if (cpdResult.ok) {
      annotateDuplicateFragments(cpdResult.fragments, area, changedFiles);
      allFragments.push(...cpdResult.fragments);
      console.log(`    Found ${cpdResult.fragments.length} duplicate fragments`);
    } else if (cpdResult.skipped) {
      console.log(`  ⚠️  CPD ${area}: ${cpdResult.error} (skipped)`);
    } else {
      console.log(`  ⚠️  CPD ${area} error: ${cpdResult.error}`);
    }
  }

  metrics.duplicateCode = allFragments;
  updateDuplicateCounts(metrics, allFragments);

  console.log(`  CPD total: ${allFragments.length} duplicate fragments`);

  writeFileSync(
    join(rawDir, "cpd-fragments.json"),
    JSON.stringify(metrics.duplicateCode, null, 2),
    "utf8"
  );
}

function scanLizardBatches({ targetFiles, root, config }) {
  const maxFilesPerBatch = 200;
  const allFunctions = [];
  const errors = [];

  for (let i = 0; i < targetFiles.length; i += maxFilesPerBatch) {
    const batch = targetFiles.slice(i, i + maxFilesPerBatch);
    const batchIdx = Math.floor(i / maxFilesPerBatch) + 1;
    const totalBatches = Math.ceil(targetFiles.length / maxFilesPerBatch);

    const lizardResult = scanWithLizard({
      files: batch,
      cwd: root,
      toolConfig: config.tools.lizard
    });

    if (lizardResult.ok) {
      allFunctions.push(...lizardResult.functions);
    } else {
      errors.push(`batch ${batchIdx}/${totalBatches}: ${lizardResult.error}`);
    }

    if (totalBatches > 1 || !lizardResult.ok) {
      console.log(`  Lizard batch ${batchIdx}/${totalBatches}: ${lizardResult.ok ? "ok" : "error"}`);
    }
  }

  return { functions: allFunctions, errors };
}

function buildFileAreaAggregates(fileMetrics, config) {
  const areaAggMap = new Map();

  for (const file of fileMetrics) {
    const existing = areaAggMap.get(file.codeArea);
    if (existing) {
      existing.files++;
      existing.lines += file.lines;
    } else {
      const areaDef = config.codeAreas[file.codeArea];
      areaAggMap.set(file.codeArea, {
        codeArea: file.codeArea,
        files: 1,
        lines: file.lines,
        functions: 0,
        warningPolicy: areaDef?.warningPolicy || "moderate"
      });
    }
  }

  return Array.from(areaAggMap.values()).sort((a, b) => b.lines - a.lines);
}

function updateFunctionCounts(metrics) {
  const funcByArea = new Map();
  for (const func of metrics.functionMetrics) {
    funcByArea.set(func.codeArea, (funcByArea.get(func.codeArea) || 0) + 1);
  }
  for (const agg of metrics.aggregates.byCodeArea) {
    agg.functions = funcByArea.get(agg.codeArea) || 0;
  }
  metrics.aggregates.overall.totalFunctions = metrics.functionMetrics.length;
}

function updateDuplicateCounts(metrics, allFragments) {
  const dupByArea = new Map();
  for (const dup of allFragments) {
    for (const area of dup.codeAreas) {
      dupByArea.set(area, (dupByArea.get(area) || 0) + 1);
    }
  }
  for (const agg of metrics.aggregates.byCodeArea) {
    agg.duplicateFragments = dupByArea.get(agg.codeArea) || 0;
  }
}

function annotateDuplicateFragments(fragments, area, changedFiles) {
  for (const frag of fragments) {
    for (const loc of frag.locations) {
      loc.codeArea = area;
    }
    frag.codeAreas = [area];
    frag.hitsChangedScope = frag.locations.some((l) => isInChangedScope(l.path, changedFiles));
  }
}

function isToolAvailable(toolResults, name) {
  return toolResults.find((t) => t.name === name)?.available;
}

function isInChangedScope(path, changedFiles) {
  return changedFiles.some((changedFile) => path.includes(changedFile) || changedFile.includes(path));
}
