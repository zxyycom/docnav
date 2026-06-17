/**
 * Baseline revision quality scan runner.
 */

import { scanWithLizard } from "./tools/lizard.mjs";
import { scanWithScc } from "./tools/scc.mjs";
import { scanWithCpd } from "./tools/cpd.mjs";
import { classifyFile, classifyFiles, isExcluded } from "./classify.mjs";
import { buildAggregates } from "./aggregate.mjs";
import { collectBaselineFiles, buildFingerprints } from "./files.mjs";

/**
 * 在 materialized baseline 目录中运行当前工具扫描。
 *
 * 只收集 fingerprints 和 baseline 指标明细用于趋势与 warning delta；
 * 不生成 baseline warnings。
 *
 * @param {string} workDir - Materialized baseline work directory
 * @param {Array} toolResults - Tool availability results
 * @param {object} config - Quality config
 * @returns {{ fingerprints: object, fileMetrics: Array, functionMetrics: Array, duplicateCode: Array, aggregates: object }}
 */
export function scanBaselineRevision(workDir, toolResults, config) {
  const baselineFiles = collectBaselineFiles(workDir, config);
  const fileMap = classifyFiles(baselineFiles, config.codeAreas, config.generatedFiles);
  const fingerprints = buildFingerprints(fileMap, workDir);

  let fileMetrics = [];
  let functionMetrics = [];
  let duplicateCode = [];
  let byLanguage = [];

  if (isToolAvailable(toolResults, "scc")) {
    ({ fileMetrics, byLanguage } = scanBaselineScc({ workDir, baselineFiles, config }));
  }

  if (isToolAvailable(toolResults, "lizard")) {
    functionMetrics = scanBaselineLizard({ workDir, baselineFiles, config });
  }

  if (isToolAvailable(toolResults, "pmd-cpd")) {
    duplicateCode = scanBaselineCpd({ workDir, fileMap, config });
  }

  const aggregates = buildAggregates({
    fileMetrics,
    functionMetrics,
    duplicateCode,
    byLanguage,
    config
  });

  return { fingerprints, fileMetrics, functionMetrics, duplicateCode, aggregates };
}

function scanBaselineScc({ workDir, baselineFiles, config }) {
  console.log("  Running baseline scc...");
  const sccResult = scanWithScc({
    cwd: workDir,
    includePaths: baselineFiles,
    excludeDirs: config.excludeDirs,
    toolConfig: config.tools.scc
  });

  if (!sccResult.ok) {
    throw new Error(`baseline scc scan failed: ${sccResult.error}`);
  }

  for (const file of sccResult.files) {
    file.codeArea = classifyFile(file.path, config.codeAreas, config.generatedFiles);
    file.isChanged = false;
  }

  const fileMetrics = sccResult.files.filter(
    (f) => !isExcluded(f.path, config.excludeDirs, config.generatedFiles)
  );

  console.log(`    Baseline scc: ${fileMetrics.length} files`);
  return { fileMetrics, byLanguage: sccResult.aggregates.byLanguage };
}

function scanBaselineLizard({ workDir, baselineFiles, config }) {
  console.log("  Running baseline Lizard...");
  const targetFiles = baselineFiles.filter(
    (f) => (f.endsWith(".rs") || f.endsWith(".mjs") || f.endsWith(".js")) &&
      !isExcluded(f, config.excludeDirs, config.generatedFiles)
  );
  const lizardResult = scanWithLizard({
    files: targetFiles,
    cwd: workDir,
    toolConfig: config.tools.lizard
  });

  if (!lizardResult.ok) {
    throw new Error(`baseline lizard scan failed: ${lizardResult.error}`);
  }

  for (const func of lizardResult.functions) {
    func.codeArea = classifyFile(func.file, config.codeAreas, config.generatedFiles);
    func.isChanged = false;
  }

  const functionMetrics = lizardResult.functions.filter(
    (f) => !isExcluded(f.file, config.excludeDirs, config.generatedFiles)
  );
  console.log(`    Baseline Lizard: ${functionMetrics.length} functions`);
  return functionMetrics;
}

function scanBaselineCpd({ workDir, fileMap, config }) {
  console.log("  Running baseline PMD CPD...");
  const fragments = [];

  for (const [area, areaFiles] of fileMap.entries()) {
    const targetFiles = areaFiles.filter(
      (f) => !isExcluded(f, config.excludeDirs, config.generatedFiles)
    );

    if (targetFiles.length < 2) {
      continue;
    }

    const minTokens = config.pmdCpd.minimumTokens[area] ?? config.pmdCpd.defaultMinimumTokens;
    const cpdResult = scanWithCpd({
      files: targetFiles,
      cwd: workDir,
      toolConfig: config.tools.pmdCpd,
      minimumTokens: minTokens,
      codeArea: area,
      skipIfUnavailable: true
    });

    if (!cpdResult.ok && !cpdResult.skipped) {
      throw new Error(`baseline CPD scan failed: ${cpdResult.error}`);
    }

    if (cpdResult.ok) {
      annotateBaselineDuplicates(cpdResult.fragments, area);
      fragments.push(...cpdResult.fragments);
    }
  }

  console.log(`    Baseline CPD: ${fragments.length} duplicate fragments`);
  return fragments;
}

function annotateBaselineDuplicates(fragments, area) {
  for (const frag of fragments) {
    for (const loc of frag.locations) {
      loc.codeArea = area;
    }
    frag.codeAreas = [area];
    frag.hitsChangedScope = false;
  }
}

function isToolAvailable(toolResults, name) {
  return toolResults.find((t) => t.name === name)?.available;
}
