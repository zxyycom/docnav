/**
 * Code quality trend delta generation.
 */

/**
 * 生成趋势 delta，比较当前指标与 baseline snapshot。
 *
 * @param {import('./schema.mjs').QualityMetrics} metrics
 * @param {{ fingerprints: object, fileMetrics: Array|null, aggregates: object|null }} baselineSnapshot
 * @returns {import('./schema.mjs').TrendDelta[]}
 */
export function generateTrends(metrics, baselineSnapshot) {
  /** @type {import('./schema.mjs').TrendDelta[]} */
  const trends = [];

  const current = metrics.aggregates.overall || {};
  const baseline = baselineSnapshot.aggregates?.overall || {};

  const overallMetrics = [
    ["total-files", "totalFiles", "files"],
    ["total-lines", "totalLines", "lines"],
    ["total-code-lines", "totalCodeLines", "lines"],
    ["total-file-complexity", "totalFileComplexity", "complexity"],
    ["total-functions", "totalFunctions", "functions"],
    ["total-function-lines", "totalFunctionLines", "lines"],
    ["total-function-parameters", "totalFunctionParameters", "parameters"],
    ["total-function-cyclomatic-complexity", "totalFunctionCyclomaticComplexity", "complexity"],
    ["duplicate-fragments", "totalDuplicateFragments", "fragments"]
  ];

  for (const [metric, field, unit] of overallMetrics) {
    trends.push(makeTrend(metric, current[field] ?? null, baseline[field] ?? null, unit));
  }

  if (baselineSnapshot.aggregates?.byLanguage) {
    appendLanguageTrends(trends, metrics, baselineSnapshot, current, baseline);
  }

  if (baselineSnapshot.aggregates?.byCodeArea) {
    appendCodeAreaTrends(trends, metrics, baselineSnapshot);
  }

  if (baselineSnapshot.fingerprints) {
    appendFingerprintTrends(trends, metrics, baselineSnapshot);
  }

  return trends;
}

function appendLanguageTrends(trends, metrics, baselineSnapshot, current, baseline) {
  const baselineLanguages = new Map();
  for (const lang of baselineSnapshot.aggregates.byLanguage) {
    baselineLanguages.set(lang.language, lang);
  }

  for (const lang of (metrics.aggregates.byLanguage || [])) {
    const baselineLanguage = baselineLanguages.get(lang.language);
    if (!baselineLanguage) continue;

    trends.push(makeTrend(`lang-${lang.language}-files`, lang.files, baselineLanguage.files, "files"));
    trends.push(makeTrend(`lang-${lang.language}-lines`, lang.lines, baselineLanguage.lines, "lines"));
    trends.push(makeTrend(
      `lang-${lang.language}-share`,
      percentOf(lang.codeLines || 0, current.totalCodeLines || 0),
      percentOf(baselineLanguage.codeLines || 0, baseline.totalCodeLines || 0),
      "percent"
    ));
  }
}

function appendCodeAreaTrends(trends, metrics, baselineSnapshot) {
  const baselineAreas = new Map();
  for (const area of baselineSnapshot.aggregates.byCodeArea) {
    baselineAreas.set(area.codeArea, area);
  }

  const areaMetrics = [
    ["files", "files", "files"],
    ["lines", "lines", "lines"],
    ["code-lines", "codeLines", "lines"],
    ["file-complexity", "fileComplexity", "complexity"],
    ["functions", "functions", "functions"],
    ["function-lines", "functionLines", "lines"],
    ["function-parameters", "parameterCount", "parameters"],
    ["function-cyclomatic-complexity", "cyclomaticComplexity", "complexity"],
    ["duplicate-fragments", "duplicateFragments", "fragments"]
  ];

  for (const area of (metrics.aggregates.byCodeArea || [])) {
    const baselineArea = baselineAreas.get(area.codeArea);
    if (!baselineArea) continue;

    for (const [metric, field, unit] of areaMetrics) {
      trends.push(makeTrend(
        `area-${area.codeArea}-${metric}`,
        area[field] ?? null,
        baselineArea[field] ?? null,
        unit
      ));
    }
  }
}

function appendFingerprintTrends(trends, metrics, baselineSnapshot) {
  const currentFingerprints = metrics.currentFingerprints || {};

  for (const [area, baselineFingerprint] of Object.entries(baselineSnapshot.fingerprints)) {
    const currentFingerprint = currentFingerprints[area];
    if (!currentFingerprint) continue;

    trends.push(makeTrend(
      `area-${area}-files`,
      currentFingerprint.fileCount,
      baselineFingerprint.fileCount,
      "files"
    ));
    trends.push(makeTrend(
      `area-${area}-fingerprint-changed`,
      currentFingerprint.fingerprint !== baselineFingerprint.fingerprint ? 1 : 0,
      0,
      "boolean"
    ));
  }
}

function makeTrend(metric, current, baseline, unit) {
  const delta = (current !== null && baseline !== null) ? current - baseline : null;
  const percentChange = (delta !== null && baseline !== null && baseline !== 0)
    ? Math.round((delta / baseline) * 1000) / 10
    : null;

  return { metric, current, baseline, delta, percentChange, unit };
}

function percentOf(value, total) {
  if (!total) return null;
  return Math.round((value / total) * 1000) / 10;
}
